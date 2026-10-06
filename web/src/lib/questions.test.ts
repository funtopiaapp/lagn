// The typing finds the question; the answer comes from the question chosen.
// So what must hold here is that matching is deterministic, that a plausible
// phrase finds the right matter, and that nothing is matched when nothing
// fits - a guess would produce an answer indistinguishable from a derived one.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { suggest } from "./questions";
import type { Question } from "../types";

const catalogue: Question[] = JSON.parse(
  readFileSync(join(__dirname, "../../../corpus/questions.json"), "utf8"),
).questions;

const top = (typed: string) => suggest(catalogue, typed)[0]?.question;

describe("matching a typed phrase to a reviewed question", () => {
  it("finds the matter people actually type", () => {
    expect(top("green card")?.topic).toBe("immigration");
    expect(top("citizenship")?.topic).toBe("immigration");
    expect(top("buying a house")?.topic).toBe("wealth");
    expect(top("should i change jobs")?.topic).toBe("career");
    expect(top("when to marry")?.topic).toBe("marriage");
    expect(top("having a baby")?.topic).toBe("progeny");
    expect(top("surgery")?.topic).toBe("health");
    expect(top("start a business")?.topic).toBe("career");
    expect(top("college admission")?.topic).toBe("education");
  });

  it("prefers a whole phrase over a stray word in it", () => {
    // "house" alone appears in several; the phrase settles it.
    expect(top("buy a house")?.question).toMatch(/buy a house/i);
    expect(top("sell property")?.question).toMatch(/sell property/i);
    // And "study abroad" is the foreign question, not the plain study one.
    expect(top("study abroad")?.topic).toBe("immigration");
    expect(top("start a course of study")?.topic).toBe("education");
  });

  it("is deterministic: the same text always gives the same order", () => {
    const a = suggest(catalogue, "house").map((s) => s.question.id);
    const b = suggest(catalogue, "house").map((s) => s.question.id);
    expect(a).toEqual(b);
    expect(a.length).toBeGreaterThan(0);
  });

  it("matches nothing rather than guessing", () => {
    expect(suggest(catalogue, "")).toEqual([]);
    expect(suggest(catalogue, "x")).toEqual([]);
    // Words with no bearing on anything the engine reads.
    expect(suggest(catalogue, "qwertyuiop")).toEqual([]);
    expect(suggest(catalogue, "the weather tomorrow")).toEqual([]);
  });

  it("ignores words too common to tell two questions apart", () => {
    // Nothing here but stop words, so there is nothing to match on.
    expect(suggest(catalogue, "when should i")).toEqual([]);
    expect(suggest(catalogue, "is it the right time")).toEqual([]);
  });

  it("falls back to the general question, but a specific matter still wins", () => {
    // "good time" is deliberately a keyword of the catch-all, so someone who
    // types only that gets the question that asks only that.
    expect(suggest(catalogue, "is this a good time")[0]?.question.id).toBe("q30");
    // And naming the matter outranks it.
    expect(suggest(catalogue, "is this a good time to buy a house")[0]?.question.topic).toBe("wealth");
    expect(suggest(catalogue, "good time for a wedding")[0]?.question.topic).toBe("marriage");
  });

  it("returns a bounded list, best first", () => {
    const hits = suggest(catalogue, "house property land buy sell", 6);
    expect(hits.length).toBeLessThanOrEqual(6);
    const scores = hits.map((h) => h.score);
    expect([...scores].sort((a, b) => b - a)).toEqual(scores);
  });
});

describe("the catalogue itself", () => {
  it("covers the matters people consult about, across several readings", () => {
    expect(catalogue.length).toBeGreaterThanOrEqual(25);
    const topics = new Set(catalogue.map((q) => q.topic));
    for (const t of ["marriage", "progeny", "wealth", "career", "education", "health", "immigration"]) {
      expect(topics).toContain(t);
    }
  });

  it("gives every question an id, a question mark and keywords", () => {
    const ids = new Set<string>();
    for (const q of catalogue) {
      expect(q.question.trim().endsWith("?")).toBe(true);
      expect(q.keywords.length).toBeGreaterThan(0);
      expect(ids.has(q.id)).toBe(false);
      ids.add(q.id);
      // A keyword is a match term, so it must be lowercase to match reliably.
      for (const k of q.keywords) expect(k).toBe(k.toLowerCase());
    }
  });

  it("is reachable: every question can be found by one of its own keywords", () => {
    for (const q of catalogue) {
      const first = q.keywords[0]!;
      const hits = suggest(catalogue, first, 30).map((h) => h.question.id);
      expect(hits).toContain(q.id);
    }
  });
});
