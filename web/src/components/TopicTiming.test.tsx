import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { TopicTiming } from "./TopicTiming";
import type { TimingRank, TimingWindow } from "../types";

const w = (rank: TimingRank, start: string, over: Partial<TimingWindow> = {}): TimingWindow => ({
  rule: "", start, end: `${Number(start.slice(0, 4)) + 2}${start.slice(4)}`,
  maha: "Shani", antar: "Budha", matched: [], verdict: "mixed", score: 0,
  supports: [], cautions: [], live: false, rank, when: "ahead", ...over,
});

const all: TimingWindow[] = [
  w("best", "2026-01-01", { live: true, supports: ["Supports career: a well-placed period lord"] }),
  w("good", "2029-01-01"),
  w("mixed", "2032-01-01"),
  w("caution", "2035-01-01", { cautions: ["Asks care over career: the lord sits in a dusthana"] }),
  w("not judged", "2038-01-01"),
];

describe("TopicTiming", () => {
  it("explains every rank it shows, in plain words", () => {
    render(<TopicTiming windows={all} topic="Career" ages={[18, 65]} />);
    // The legend covers each rank present, by name.
    const legend = screen.getByText(/What best, good, mixed, caution mean/);
    expect(legend).toBeInTheDocument();
    const details = legend.closest("details")!;
    expect(within(details).getByText(/One to plan around/)).toBeInTheDocument();
    expect(within(details).getByText(/A supportive stretch in general/)).toBeInTheDocument();
    expect(within(details).getByText(/Support and difficulty both apply/)).toBeInTheDocument();
    expect(within(details).getByText(/This stretch asks for care/)).toBeInTheDocument();
  });

  it("never explains a rank that is not on the page", () => {
    render(<TopicTiming windows={[w("best", "2026-01-01")]} topic="Career" ages={[18, 65]} />);
    expect(screen.getByText(/What best mean/)).toBeInTheDocument();
    expect(screen.queryByText(/This stretch asks for care/)).toBeNull();
  });

  it("puts a hint on the chip for a pointer, and the sentence on the page for everyone else", () => {
    render(<TopicTiming windows={all} topic="Career" ages={[18, 65]} />);
    // A phone has no hover, so the meaning must also be visible text.
    const chips = screen.getAllByTitle(/One to plan around|plan around/);
    expect(chips.length).toBeGreaterThan(0);
    expect(screen.getAllByText(/One to plan around\. The classical timing rules point at career/).length)
      .toBeGreaterThan(0);
  });

  it("names the area in the rank explanation, not just 'this'", () => {
    render(<TopicTiming windows={[w("caution", "2035-01-01")]} topic="Living abroad" ages={[18, 70]} />);
    expect(screen.getAllByText(/does not say living abroad will go wrong/).length).toBeGreaterThan(0);
  });

  it("leads with the recommended stretches and can show the whole span in order", async () => {
    const { default: userEvent } = await import("@testing-library/user-event");
    render(<TopicTiming windows={all} topic="Career" ages={[18, 65]} />);
    expect(screen.getByText(/2 of 5/)).toBeInTheDocument();
    // Only best + good until asked for the rest.
    expect(screen.queryByText("2035-01-01 → 2037-01-01")).toBeNull();
    await userEvent.setup().click(screen.getByRole("button", { name: /Show the whole span: all 5/ }));
    expect(screen.getByText("2035-01-01 → 2037-01-01")).toBeInTheDocument();
  });

  it("says so plainly when nothing is recommended", () => {
    render(<TopicTiming windows={[w("caution", "2035-01-01"), w("mixed", "2037-01-01")]} topic="Marriage" ages={[18, 45]} />);
    expect(screen.getByRole("note")).toHaveTextContent(/none comes out favourable/);
    expect(screen.getByRole("note")).toHaveTextContent(/not a refusal/);
  });
});
