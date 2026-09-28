import { render, screen } from "@testing-library/react";
import { TopicView } from "./TopicView";

const birth = { date: "1985-06-21", time: "14:30:00", latitude: 13, longitude: 80, utc_offset_hours: 5.5, place: "" };
const empty = { report: { topic: "marriage", mode: "production", results: [], withheld: { draft: 34 }, supporting: [], afflicting: [], cancelled: [], unknown: [], score: 0, label: "neutral" }, windows: [] };
afterEach(() => vi.unstubAllGlobals());

describe("TopicView", () => {
  it("explains the review gate instead of showing an empty reading", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(JSON.stringify(empty))));
    render(<TopicView birth={birth} reviewToken="" />);
    expect(await screen.findByText(/No reviewed interpretations are available/)).toBeInTheDocument();
    expect(screen.getByText(/34 rules are not shown/)).toBeInTheDocument();
    // Never claims a review that did not happen.
    expect(screen.queryByText(/practising astrologer/)).toBeNull();
  });

  it("marks every draft and shows noted rules in review mode", async () => {
    const rule = (id: string, polarity: number) => ({ id, title: id, status: "draft", reviewer: null, polarity, outcome: "true", effective: true, cancelled_by: null, text: "t", trace: [] });
    const review = { ...empty, report: { ...empty.report, mode: "review", results: [rule("m.a", 2), rule("m.b", 0), rule("m.t", 0)], withheld: {}, supporting: ["m.a"], label: "supportive", score: 2 },
      windows: [{ rule: "m.t", start: "2003-06-22", end: "2003-11-06", maha: "Budha", antar: "Kuja", matched: ["Kuja"] }] };
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(JSON.stringify(review))));
    render(<TopicView birth={birth} reviewToken="tok" />);
    await screen.findByText("Supporting");
    expect(screen.getAllByText("DRAFT")).toHaveLength(2);
    expect(screen.getByText("Noted (not scored)")).toBeInTheDocument();
    expect(screen.queryByText("m.t", { selector: "strong" })).toBeNull(); // timing-only rule shown as periods, not noted
    expect(screen.getByText("2003-06-22")).toBeInTheDocument();
  });
});

describe("TopicView provenance", () => {
  it("states who reviewed the rules shown", async () => {
    const r = { id: "m.a", title: "m.a", status: "approved", reviewer: "Claude (AI review at the product owner's direction)", polarity: 2, outcome: "true", effective: true, cancelled_by: null, text: "t", trace: [] };
    const body = { ...empty, report: { ...empty.report, results: [r], withheld: {}, supporting: ["m.a"], label: "supportive", score: 2 } };
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(JSON.stringify(body))));
    render(<TopicView birth={birth} reviewToken="" />);
    expect(await screen.findByText(/Rules reviewed by: Claude \(AI review/)).toBeInTheDocument();
    expect(screen.queryByText("DRAFT")).toBeNull();
  });
});

describe("TopicView write-up", () => {
  it("reads on open, shows the write-up first and sends the sex given once on the birth form", async () => {
    const r = { id: "career.l10.strong", title: "The 10th lord is strong", status: "approved", reviewer: "r", polarity: 2, outcome: "true", effective: true, cancelled_by: null, text: "t", trace: [] };
    const writeup = {
      summary: ["The indications for career are clearly supportive.", "disclaimer text"],
      sections: [
        { heading: "The 10th house: Karma", paragraphs: ["Your 10th house, Karma bhava, governs career."] },
        { heading: "What supports", paragraphs: ["These apply:"], points: [{ rule: "career.l10.strong", title: "The 10th lord is strong", text: "t", meaning: "Work tends to go somewhere.", polarity: 2, because: ["Chandra is in its own sign"] }] },
      ],
    };
    const body = { ...empty, report: { ...empty.report, topic: "career", results: [r], withheld: {}, supporting: [r.id], label: "supportive", score: 2 }, writeup };
    const f = vi.fn().mockResolvedValue(new Response(JSON.stringify(body)));
    vi.stubGlobal("fetch", f);
    render(<TopicView birth={birth} sex="female" reviewToken="" topic={{ id: "career", title: "Career", summary: "", disclaimer: "disclaimer text", ages: [20, 65] }} />);
    expect(await screen.findByText("The indications for career are clearly supportive.")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "The 10th house: Karma" })).toBeInTheDocument();
    expect(screen.getByText("Because: Chandra is in its own sign")).toBeInTheDocument();
    // The plain-language line sits under the finding, labelled.
    expect(screen.getAllByText("What this means for you:")[0]).toBeInTheDocument();
    expect(screen.getByText(/Work tends to go somewhere\./)).toBeInTheDocument();
    // The disclaimer appears once, not twice.
    expect(screen.getAllByText("disclaimer text")).toHaveLength(1);
    // The rule cards are still there, under Details.
    expect(screen.getByText(/Details: every rule checked/)).toBeInTheDocument();
    expect(screen.queryByRole("combobox")).toBeNull(); // no sex selector here
    const sent = JSON.parse((f.mock.calls[0] as [string, RequestInit])[1].body as string);
    expect(sent.sex).toBe("female");
  });
});
