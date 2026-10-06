import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PeriodsView } from "./PeriodsView";
import { ReadingsView } from "./ReadingsView";

const birth = { date: "1985-06-21", time: "14:30:00", latitude: 13, longitude: 80, utc_offset_hours: 5.5, place: "" };
afterEach(() => vi.unstubAllGlobals());

const topics = {
  topics: [
    { id: "marriage", title: "Marriage", summary: "Marriage prospects.", ages: [18, 45] },
    { id: "health", title: "Health", summary: "Tendencies.", disclaimer: "This is a traditional astrological reading, not a medical assessment. For any health concern, consult a qualified doctor.", ages: [0, 90] },
  ],
  bhavas: [],
};
const rule = (id: string, polarity: number, text = "t") => ({ id, title: id, status: "approved", reviewer: "Claude (AI review at the product owner's direction)", polarity, outcome: "true", effective: true, cancelled_by: null, text, trace: [] });
const pariharam = { pariharam: { id: "graha.saturn", title: "For Saturn (Shani)", triggers: ["graha:saturn"], deity: "Hanuman", day: "Saturday", practices: ["Light a sesame-oil lamp on Saturdays"], places: ["Thirunallar"], charity: "Donate black sesame (ellu)" }, because: ["health.l1.dusthana"] };

function route(map: Record<string, unknown>) {
  return vi.fn((url: string) => {
    const key = Object.keys(map).find((k) => url.includes(k));
    return Promise.resolve(new Response(JSON.stringify(key ? map[key] : { error: "no" }), { status: key ? 200 : 404 }));
  });
}

// Roomy margin: these tests are about the topic list, not the birth-time
// warning, which has its own tests.
const lagna = {
  rasi: "Mesha", rasi_tamil: "Mesham", degrees: "10\u00b000'00\"",
  nakshatra: "Ashwini", nakshatra_tamil: "Aswini", pada: 3,
  holds_for: { minutes_in: 90, minutes_left: 95, margin_minutes: 90, capped: false },
};

describe("ReadingsView", () => {
  it("offers every catalogue topic and shows the health disclaimer before any reading", async () => {
    vi.stubGlobal("fetch", route({ "/api/topics": topics }));
    render(<ReadingsView birth={birth} reviewToken="" lagna={lagna} />);
    const nav = await screen.findByRole("navigation", { name: "Topics" });
    expect(within(nav).getAllByRole("button").map((b) => b.textContent)).toEqual(["Marriage", "Health"]);
    await userEvent.setup().click(within(nav).getByRole("button", { name: "Health" }));
    expect(screen.getByRole("heading", { name: "Health" })).toBeInTheDocument();
    // The birth-time margin is a note too, so name the one meant here.
    const notes = screen.getAllByRole("note").map((n) => n.textContent ?? "");
    expect(notes.some((t) => /consult a qualified doctor/.test(t))).toBe(true);
  });

  it("shows pariharams with the findings that brought them up", async () => {
    const reading = { report: { topic: "health", mode: "production", results: [rule("health.l1.dusthana", -2)], withheld: {}, supporting: [], afflicting: ["health.l1.dusthana"], cancelled: [], unknown: [], score: -2, label: "afflicted" }, windows: [], meta: topics.topics[1], pariharams: [pariharam] };
    vi.stubGlobal("fetch", route({ "/api/topics": topics, "/api/topic/health": reading }));
    render(<ReadingsView birth={birth} reviewToken="" lagna={lagna} />);
    const u = userEvent.setup();
    await u.click(await screen.findByRole("button", { name: "Health" }));
    expect(await screen.findByText("For Saturn (Shani)")).toBeInTheDocument();
    expect(screen.getByText(/No gemstones or paid services/)).toBeInTheDocument();
    expect(screen.getByText("Because: health.l1.dusthana")).toBeInTheDocument();
  });

  it("never shows one topic's reading under another topic", async () => {
    const reading = { report: { topic: "marriage", mode: "production", results: [rule("marriage.x", 1)], withheld: {}, supporting: ["marriage.x"], afflicting: [], cancelled: [], unknown: [], score: 1, label: "supportive" }, windows: [], pariharams: [] };
    vi.stubGlobal("fetch", route({ "/api/topics": topics, "/api/topic/marriage": reading }));
    render(<ReadingsView birth={birth} reviewToken="" lagna={lagna} />);
    const u = userEvent.setup();
    await screen.findByText("Supporting");
    await u.click(screen.getByRole("button", { name: "Health" }));
    expect(screen.queryByText("Supporting")).toBeNull();
  });
});

describe("PeriodsView", () => {
  const w = (maha: string, antar: string, score: number, current = false) => ({
    start: "2030-01-01", end: "2031-01-01", maha_name: maha, antar_name: antar, current, score, sensitive: score <= -2,
    amplifiers: score < 0 ? [rule("periods.antar.dusthana", -1)] : [], negators: [], noted: [], cancelled: [],
    focus: [{ house: 7, name: "Kalatra", significations: ["spouse and marriage"], via: ["occupies"] }],
    pressures: [], supports: [], pariharams: score <= -2 ? [pariharam] : [],
    explanation: [`${maha} mahadasha, ${antar} bhukti. The matters of this period are spouse and marriage (house 7).`, "Calls for care: the sub-period lord sits in a dusthana."],
  });
  const data = { mode: "production", withheld: {}, moon_sign: "Mesha", transits: [], windows: [w("Shani", "Shani", 0, true), w("Shani", "Ketu", -2), w("Shani", "Shukra", 1)] };

  it("lists every window, marks sensitive and current ones, and filters", async () => {
    const f = route({ "/api/periods": data });
    vi.stubGlobal("fetch", f);
    render(<PeriodsView birth={birth} reviewToken="" />);
    const u = userEvent.setup();
    await u.click(screen.getByRole("button", { name: "Show periods" }));
    expect(await screen.findByText(/3 periods from the Moon sign Mesha; 1 marked sensitive/)).toBeInTheDocument();
    expect(screen.getAllByText("SENSITIVE")).toHaveLength(1);
    expect(screen.getAllByText("NOW")).toHaveLength(1);
    await u.click(screen.getByRole("checkbox"));
    const items = document.querySelectorAll("li.period-window");
    expect([...items].map((i) => i.getAttribute("data-window"))).toEqual(["Shani/Shani", "Shani/Ketu"]);
    const body = JSON.parse((f.mock.calls[0] as unknown as [string, RequestInit])[1].body as string);
    expect(body).toMatchObject({ from_age: 18, to_age: 70, mode: "production" });
  });

  it("refuses an impossible age range without calling the server", async () => {
    const f = route({});
    vi.stubGlobal("fetch", f);
    render(<PeriodsView birth={birth} reviewToken="" />);
    const u = userEvent.setup();
    const from = screen.getByLabelText("From age");
    await u.clear(from); await u.type(from, "80");
    expect(screen.getByRole("button", { name: "Show periods" })).toBeDisabled();
    expect(f).not.toHaveBeenCalled();
  });
});

describe("asking a question from the readings page", () => {
  // The reported bug: picking a question about marriage produced no answer.
  // Marriage is the default topic, so the topic view never remounted, never
  // refetched, and never reported the reading back - and the code cleared the
  // reading it already had.
  const withQuestions = {
    ...topics,
    questions: [
      { id: "q01", question: "Is this a good time to marry?", topic: "marriage", keywords: ["marry", "marriage"] },
      { id: "q07", question: "Is this a good time to buy a house?", topic: "health", keywords: ["house"] },
    ],
  };
  const marriageReading = {
    report: {
      topic: "marriage", mode: "production", results: [rule("marriage.x", 2)], withheld: {},
      supporting: ["marriage.x"], afflicting: [], cancelled: [], unknown: [], score: 2, label: "supportive",
    },
    windows: [{
      rule: "marriage.timing.periods", start: "2030-01-01", end: "2031-06-01",
      maha: "Shukra", antar: "Budha", matched: ["Shukra"], verdict: "favourable", score: 2,
      supports: ["Supports marriage: a well-placed period lord"], cautions: [],
      live: true, rank: "best", when: "ahead",
    }],
    meta: topics.topics[0], pariharams: [], writeup: null,
  };

  it("answers a question about the topic already selected", async () => {
    vi.stubGlobal("fetch", route({ "/api/topics": withQuestions, "/api/topic/marriage": marriageReading }));
    render(<ReadingsView birth={birth} reviewToken="" lagna={lagna} />);
    const u = userEvent.setup();

    await u.type(await screen.findByLabelText(/What is it about/), "marriage");
    await u.click(await screen.findByRole("button", { name: /Is this a good time to marry\?/ }));

    // The prepared answer appears, with the question as its heading.
    expect(await screen.findByRole("heading", { name: "Is this a good time to marry?" })).toBeInTheDocument();
    expect(screen.getByText(/Taken as a whole, your chart supports marriage/)).toBeInTheDocument();
    expect(screen.getByText(/January 2030 to June 2031/)).toBeInTheDocument();
  });

  it("says nothing matched rather than answering a different question", async () => {
    vi.stubGlobal("fetch", route({ "/api/topics": withQuestions, "/api/topic/marriage": marriageReading }));
    render(<ReadingsView birth={birth} reviewToken="" lagna={lagna} />);
    await userEvent.setup().type(await screen.findByLabelText(/What is it about/), "qwertyuiop");
    expect(screen.getByText(/No reviewed question matches that/)).toBeInTheDocument();
  });
});
