import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { FamilyView } from "./FamilyView";

const birth = { date: "1985-06-21", time: "14:30:00", latitude: 13, longitude: 80, utc_offset_hours: 5.5, place: "" };
const child = { id: "c1", name: "Meena", relation: "child", birth: { date: "2012-04-02", time: "09:15:00", latitude: 13.08, longitude: 80.27, utc_offset_hours: 5.5, place: "Chennai" } };
const rule = (id: string, polarity: number) => ({ id, title: `title ${id}`, status: "approved", reviewer: "r", polarity, outcome: "true", effective: true, cancelled_by: null, text: "t", trace: [] });
const report = (topic: string, score: number, results: ReturnType<typeof rule>[]) => ({ topic, mode: "production", results, withheld: {}, supporting: results.filter((r) => r.polarity > 0).map((r) => r.id), afflicting: results.filter((r) => r.polarity < 0).map((r) => r.id), cancelled: [], unknown: [], score, label: "mixed" });
const reading = {
  relation: "child",
  relational: { topic: "progeny", house: 5, supporting: [rule("progeny.jupiter_aspects_5", 2)], afflicting: [], score: 2, lean: "favourable",
    writeup: { summary: ["The indications for children are clearly supportive."], sections: [{ heading: "The 5th house: Putra", paragraphs: ["Your 5th house text"] }] } },
  own: [
    { meta: { id: "health", title: "Health", summary: "", disclaimer: "This is a traditional astrological reading, not a medical assessment. For any health concern, consult a qualified doctor.", ages: [0, 90] }, report: report("health", -1, [rule("health.h1.malefic", -1)]), lean: "calls_for_care", pariharams: [],
      writeup: { summary: ["On balance, health calls for care and effort."], sections: [{ heading: "The 1st house: Tanu", paragraphs: ["Meena's lagna text"] }] } },
    { meta: { id: "education", title: "Education", summary: "", ages: [4, 30] }, report: report("education", 1, [rule("education.jupiter_aspects", 1)]), lean: "favourable", pariharams: [], writeup: null },
  ],
  agreement: "differ",
  compared_with: "health",
  comparison: [
    "Two charts are read here, and they answer different questions. {member}'s own chart shows their life, which is theirs and not yours.",
    "Your chart, read for children: favourable.",
    "{member}'s own chart, read for health: calls for care.",
  ],
};
afterEach(() => { localStorage.clear(); vi.unstubAllGlobals(); });

describe("FamilyView", () => {
  it("reads a stored member, sending both births, and shows the verdict then the three tabs", async () => {
    localStorage.setItem("lagn.family.v1", JSON.stringify([child]));
    const f = vi.fn().mockResolvedValue(new Response(JSON.stringify(reading)));
    vi.stubGlobal("fetch", f);
    render(<FamilyView birth={birth} sex="male" reviewToken="" />);
    const u = userEvent.setup();
    await u.click(screen.getByRole("button", { name: "Read" }));

    // Verdict first: who, whether they agree, and both leans with whose chart each is.
    expect(await screen.findByText(/The two readings point different ways/)).toBeInTheDocument();
    expect(screen.getByText("Meena · child")).toBeInTheDocument();
    expect(screen.getByText("Your chart", { selector: ".whose" })).toBeInTheDocument();
    expect(screen.getByText("Favourable")).toBeInTheDocument();
    expect(screen.getByText("Meena's own chart", { selector: ".whose" })).toBeInTheDocument();
    expect(screen.getByText("Calls for care")).toBeInTheDocument();
    expect(screen.getAllByText(/read for children \(5th house\)/).length).toBeGreaterThan(0);

    // Your chart is shown first; the other panes are behind their tabs.
    expect(screen.getByText("Your 5th house text")).toBeInTheDocument();
    expect(screen.queryByText("Meena's lagna text")).toBeNull();
    await u.click(screen.getByRole("button", { name: "Meena's chart" }));
    expect(screen.getByText("Meena's lagna text")).toBeInTheDocument();
    expect(screen.getByText(/Health — Calls for care/)).toBeInTheDocument();
    expect(screen.getByText(/Education — Favourable/)).toBeInTheDocument();

    // Together: the engine's own text, with the name filled in on the device.
    await u.click(screen.getByRole("button", { name: "Together" }));
    expect(screen.getByText(/Meena's own chart shows their life/)).toBeInTheDocument();
    expect(screen.queryByText(/\{member\}/)).toBeNull();

    const body = JSON.parse((f.mock.calls[0] as [string, RequestInit])[1].body as string);
    expect(body).toMatchObject({ native: { birth, sex: "male" }, member: { birth: child.birth }, relation: "child", mode: "production" });
    expect(JSON.stringify(body)).not.toContain("Meena"); // names stay on the device
    expect(screen.queryByText(/Your sex/)).toBeNull(); // asked once, on the birth form
  });

  it("removes a member only after confirmation", async () => {
    localStorage.setItem("lagn.family.v1", JSON.stringify([child]));
    const confirm = vi.fn().mockReturnValueOnce(false).mockReturnValueOnce(true);
    vi.stubGlobal("confirm", confirm);
    render(<FamilyView birth={birth} reviewToken="" />);
    const u = userEvent.setup();
    await u.click(screen.getByRole("button", { name: "Remove" }));
    expect(screen.getByText("Meena")).toBeInTheDocument();
    await u.click(screen.getByRole("button", { name: "Remove" }));
    expect(screen.queryByText("Meena")).toBeNull();
    expect(JSON.parse(localStorage.getItem("lagn.family.v1")!)).toEqual([]);
  });

  it("imports a valid export and reports an invalid one", async () => {
    render(<FamilyView birth={birth} reviewToken="" />);
    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    const u = userEvent.setup();
    await u.upload(input, new File([JSON.stringify({ format: "lagn-family/1", members: [child] })], "f.json", { type: "application/json" }));
    expect(await screen.findByText("Meena")).toBeInTheDocument();
    await u.upload(input, new File(["{}"], "g.json", { type: "application/json" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(/not a Lagn family export/);
  });
});
