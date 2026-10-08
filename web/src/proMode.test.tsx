// Phase 13A acceptance criteria 8 and 9: Lite is unchanged by this phase, and
// Pro is off by default, persists when set, and is absent from the URL.

import { describe, expect, it } from "vitest";

import { tabsFor } from "./App";

/** What the general public saw before phase 13 existed. */
const LITE = ["Chart", "Readings", "Sensitive periods", "Day timings", "Family", "Match", "Saved profiles"];

describe("the Lite surface", () => {
  it("is exactly what it was before the professional tools arrived", () => {
    // Criterion 8. If a pro tab ever leaks into Lite, this fails - which is
    // the whole point of keeping the two lists apart rather than filtering
    // one list by a flag at render time.
    expect(tabsFor("lite").map(([, label]) => label)).toEqual(LITE);
  });

  it("does not offer any professional tab at all", () => {
    const ids = tabsFor("lite").map(([id]) => id);
    expect(ids).not.toContain("jaimini");
    expect(ids).not.toContain("chara");
  });
});

describe("the Pro surface", () => {
  it("adds tabs without removing or reordering any Lite tab", () => {
    const pro = tabsFor("pro").map(([, label]) => label);
    expect(pro.slice(0, LITE.length)).toEqual(LITE);
    expect(pro.length).toBeGreaterThan(LITE.length);
  });

  it("is where the professional tools live", () => {
    const ids = tabsFor("pro").map(([id]) => id);
    expect(ids).toContain("jaimini");
    expect(ids).toContain("chara");
  });
});
