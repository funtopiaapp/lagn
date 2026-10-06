// A prepared answer has to answer the question that was asked. For "when
// should I buy a house" that means stretches still to come, soonest first -
// not the best stretch of a life that ended twenty years ago.

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { QuestionAnswer } from "./QuestionAnswer";
import type { Question, TimingRank, TimingWindow, TopicMeta, TopicResponse } from "../types";

const q: Question = { id: "q07", question: "Is this a good time to buy a house?", topic: "wealth", keywords: [] };
const topic: TopicMeta = { id: "wealth", title: "Wealth", summary: "", ages: [18, 80] } as TopicMeta;

const w = (rank: TimingRank, start: string, when: TimingWindow["when"]): TimingWindow => ({
  rule: "", start, end: `${Number(start.slice(0, 4)) + 1}${start.slice(4)}`,
  maha: "Shani", antar: "Budha", matched: [], verdict: "favourable", score: 2,
  supports: [`Supports wealth: a well-placed period lord (${start})`], cautions: [],
  live: true, rank, when,
});

const data = (windows: TimingWindow[], score = 4): TopicResponse =>
  ({
    report: { topic: "wealth", mode: "production", results: [{ id: "x" }], withheld: {},
      supporting: ["a"], afflicting: [], cancelled: [], unknown: [], score, label: "supportive" },
    windows, pariharams: [], writeup: null,
  }) as unknown as TopicResponse;

describe("a prepared answer", () => {
  it("names only the stretches still to come, soonest first", () => {
    render(<QuestionAnswer question={q} topic={topic} data={data([
      w("best", "2001-03-07", "past"),
      w("good", "2035-01-01", "ahead"),
      w("best", "2029-01-01", "ahead"),
    ])} />);
    expect(screen.getByText(/2 stretches are recommended, soonest first/)).toBeInTheDocument();
    const items = screen.getAllByRole("listitem").map((li) => li.textContent ?? "");
    expect(items[0]).toContain("2029-01-01");
    expect(items[1]).toContain("2035-01-01");
    // The past one is not offered as advice.
    expect(items.join()).not.toContain("2001-03-07");
  });

  it("puts a stretch running now before the ones ahead, and says so", () => {
    render(<QuestionAnswer question={q} topic={topic} data={data([
      w("best", "2030-01-01", "ahead"),
      w("good", "2026-01-01", "now"),
    ])} />);
    const items = screen.getAllByRole("listitem").map((li) => li.textContent ?? "");
    expect(items[0]).toContain("2026-01-01");
    expect(items[0]).toContain("running now");
  });

  it("says plainly when every recommended stretch has already passed", () => {
    render(<QuestionAnswer question={q} topic={topic} data={data([
      w("best", "2001-03-07", "past"),
      w("good", "2005-01-01", "past"),
    ])} />);
    expect(screen.getByText(/have already passed/)).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: /all now past/ })).toBeInTheDocument();
    // And it does not pretend an old date is advice.
    expect(screen.getByText(/rather than being shown an old date as though it were advice/)).toBeInTheDocument();
  });

  it("says there is no window rather than inventing one", () => {
    render(<QuestionAnswer question={q} topic={topic} data={data([
      { ...w("caution", "2030-01-01", "ahead"), rank: "caution", verdict: "asks for care" },
    ])} />);
    expect(screen.getByText(/no window to point you at/)).toBeInTheDocument();
    expect(screen.getByText(/That is not a refusal/)).toBeInTheDocument();
  });

  it("leads with the standing of the matter, and carries the topic's disclaimer", () => {
    const withDisclaimer = { ...topic, disclaimer: "Not legal advice." };
    render(<QuestionAnswer question={q} topic={withDisclaimer} data={data([w("best", "2030-01-01", "ahead")], -3)} />);
    expect(screen.getByText(/The chart asks for care over wealth/)).toBeInTheDocument();
    expect(screen.getByRole("note")).toHaveTextContent("Not legal advice.");
  });

  it("points at the day-level layer for choosing an actual day", () => {
    render(<QuestionAnswer question={q} topic={topic} data={data([w("best", "2030-01-01", "ahead")])} />);
    expect(screen.getByText(/Day timings/)).toBeInTheDocument();
  });
});
