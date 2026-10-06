// A prepared answer has to answer the question that was asked.
//
// Two things this got wrong and now pins. It offered the next clean window
// twenty years out while skipping a +2 stretch in between, because a
// seventeen-year mahadasha whose lord rules the 3rd and 6th puts a caution on
// every sub-period inside it so none of them can ever be "recommended". And
// it said "all past" without saying that the reading's own age range had run
// out, which put on the chart something that was only the range.

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { QuestionAnswer } from "./QuestionAnswer";
import type { Question, TimingRank, TimingWindow, TopicMeta, TopicResponse } from "../types";

const q: Question = { id: "q07", question: "Is this a good time to buy a house?", topic: "wealth", keywords: [] };
const topic = { id: "wealth", title: "Wealth", summary: "", ages: [18, 80] } as unknown as TopicMeta;

const w = (
  rank: TimingRank,
  start: string,
  when: TimingWindow["when"],
  over: Partial<TimingWindow> = {},
): TimingWindow => ({
  rule: "", start, end: `${Number(start.slice(0, 4)) + 1}${start.slice(4)}`,
  maha: "Budha", antar: "Surya", matched: [], verdict: "favourable", score: 2,
  supports: [`Supports wealth: a well-placed period lord`], cautions: [],
  live: true, rank, when, ...over,
});

const data = (windows: TimingWindow[], score = 4, ages: [number, number] = [18, 80]): TopicResponse =>
  ({
    report: { topic: "wealth", mode: "production", results: [{ id: "x" }], withheld: {},
      supporting: ["a"], afflicting: [], cancelled: [], unknown: [], score, label: "supportive" },
    windows, pariharams: [], writeup: null, meta: { ...topic, ages },
  }) as unknown as TopicResponse;

const dates = () => screen.getAllByRole("listitem").map((li) => li.textContent ?? "");

describe("when the chart offers clean windows ahead", () => {
  it("names only those, soonest first, and ignores the past ones", () => {
    render(<QuestionAnswer question={q} topic={topic} data={data([
      w("best", "2001-03-07", "past"),
      w("good", "2035-01-01", "ahead"),
      w("best", "2029-01-01", "ahead"),
    ])} />);
    expect(screen.getByText(/2 stretches ahead come out recommended/)).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "When to aim for" })).toBeInTheDocument();
    const d = dates();
    expect(d[0]).toContain("January 2029");
    expect(d[1]).toContain("January 2035");
    expect(d.join()).not.toContain("March 2001");
  });

  it("puts a stretch running now first and says so", () => {
    render(<QuestionAnswer question={q} topic={topic} data={data([
      w("best", "2030-01-01", "ahead"),
      w("good", "2026-01-01", "now"),
    ])} />);
    expect(dates()[0]).toContain("running now");
    expect(dates()[0]).toContain("January 2026");
  });
});

describe("when nothing ahead is clean", () => {
  // The reported case: Mercury's mahadasha, every sub-period carrying the
  // major lord's caution, with one +2 stretch inside it.
  const mercuryMaha = [
    w("mixed", "2029-03-31", "ahead", { score: -1, verdict: "asks for care",
      cautions: ["Asks care over wealth: the major-period lord rules difficult houses"] }),
    w("mixed", "2035-06-25", "ahead", { score: 2,
      cautions: ["Asks care over wealth: the major-period lord rules difficult houses"] }),
    w("mixed", "2037-09-30", "ahead", { score: 0, cautions: ["Asks care over wealth: so and so"] }),
    w("good", "2046-08-28", "ahead", { score: 3 }),
  ];

  it("offers the best-supported stretch ahead rather than skipping years to the next clean one", () => {
    // Drop the clean 2046 window: with none ahead, the +2 must be offered.
    render(<QuestionAnswer question={q} topic={topic} data={data(mercuryMaha.slice(0, 3))} />);
    expect(screen.getByText(/No stretch ahead comes out recommended outright/)).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: /best-supported stretches ahead/ })).toBeInTheDocument();
    const d = dates();
    expect(d).toHaveLength(1);
    expect(d[0]).toContain("June 2035");
    // Its cautions come with it, so the offer is not oversold.
    expect(d[0]).toContain("major-period lord rules difficult houses");
  });

  it("still prefers a clean window when there is one", () => {
    render(<QuestionAnswer question={q} topic={topic} data={data(mercuryMaha)} />);
    expect(screen.getByRole("heading", { name: "When to aim for" })).toBeInTheDocument();
    expect(dates()[0]).toContain("August 2046");
  });

  it("also offers what is supported before that clean window, rather than only the far one", () => {
    // The reported complaint: everything answered 2046 while a +2 stretch sat
    // in 2035. Both are shown now, each for what it is.
    render(<QuestionAnswer question={q} topic={topic} data={data(mercuryMaha)} />);
    expect(screen.getByRole("heading", { name: /Sooner, though not without caution/ })).toBeInTheDocument();
    const all = dates().join("\n");
    expect(all).toContain("August 2046");
    expect(all).toContain("June 2035");
    // And it says why the nearer one is not a recommendation.
    expect(screen.getByText(/not free of difficulty/)).toBeInTheDocument();
  });

  it("offers nothing sooner when the clean window is already the soonest", () => {
    render(<QuestionAnswer question={q} topic={topic} data={data([
      w("best", "2027-01-01", "ahead", { score: 3 }),
      w("mixed", "2035-06-25", "ahead", { score: 2, cautions: ["Asks care over wealth: so and so"] }),
    ])} />);
    expect(screen.queryByText(/Sooner, though not without caution/)).toBeNull();
  });
});

describe("when everything is behind", () => {
  it("blames the reading's age range when that is what ran out", () => {
    // Marriage is read to age 45; for a 1981 birth that ended in 2026.
    const marriage = { ...topic, id: "marriage", title: "Marriage", ages: [18, 45] } as unknown as TopicMeta;
    render(<QuestionAnswer
      question={{ ...q, question: "Is this a good time to marry?", topic: "marriage" }}
      topic={marriage}
      data={data([w("best", "2020-01-01", "past"), w("good", "2025-12-21", "past")], 4, [18, 45])}
    />);
    expect(screen.getByText(/covers ages 18 to 45, which for you ran out in December 2026/)).toBeInTheDocument();
    expect(screen.getByText(/range of the reading talking, not a judgement/)).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: /now behind you/ })).toBeInTheDocument();
  });

  it("says there is no supported stretch rather than inventing one", () => {
    render(<QuestionAnswer question={q} topic={topic} data={data([
      w("caution", "2030-01-01", "ahead", { score: -2, verdict: "asks for care", supports: [] }),
    ])} />);
    expect(screen.getByText(/Nothing in the range this reading covers/)).toBeInTheDocument();
    expect(screen.getByText(/That is not a refusal/)).toBeInTheDocument();
  });
});

describe("how the answer reads", () => {
  it("opens with how the chart stands, in a sentence", () => {
    render(<QuestionAnswer question={q} topic={topic} data={data([w("best", "2030-01-01", "ahead")], -3)} />);
    expect(screen.getByText(/Taken as a whole, your chart asks for care over wealth/)).toBeInTheDocument();
  });

  it("carries the topic's disclaimer and points at the day-level layer", () => {
    const withDisclaimer = { ...topic, disclaimer: "Not legal advice." } as unknown as TopicMeta;
    render(<QuestionAnswer question={q} topic={withDisclaimer} data={data([w("best", "2030-01-01", "ahead")])} />);
    expect(screen.getByRole("note")).toHaveTextContent("Not legal advice.");
    expect(screen.getByText(/Day timings/)).toBeInTheDocument();
  });

  it("gives no answer at all when no reviewed rule applies", () => {
    const empty = { ...data([]), report: { ...data([]).report, results: [] } } as TopicResponse;
    render(<QuestionAnswer question={q} topic={topic} data={empty} />);
    expect(screen.getByText(/anything said here would be invented rather than derived/)).toBeInTheDocument();
  });
});
