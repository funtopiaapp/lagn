// Phase 13A acceptance criteria 8 and 9: Lite is unchanged by this phase, and
// Pro is off by default, persists when set, and is absent from the URL.

import { afterEach, describe, expect, it, vi } from "vitest";

import { tabsFor } from "./App";

afterEach(() => vi.unstubAllEnvs());

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
    expect(ids).not.toContain("upagraha");
    expect(ids).not.toContain("kp");
    expect(ids).not.toContain("varsha");
    expect(ids).not.toContain("bala");
    expect(ids).not.toContain("yogini");
    expect(ids).not.toContain("av");
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
    expect(ids).toContain("upagraha");
    expect(ids).toContain("kp");
    expect(ids).toContain("varsha");
    expect(ids).toContain("bala");
    expect(ids).toContain("yogini");
    expect(ids).toContain("av");
  });
});

describe("when the owner has turned the professional surface off", () => {
  it("offers no professional tab even to a reader still in Pro mode", () => {
    // The switch is checked in tabsFor as well as in loadMode, so stale mode
    // held in component state cannot surface a professional tab.
    vi.stubEnv("VITE_PRO", "off");
    expect(tabsFor("pro").map(([, label]) => label)).toEqual(LITE);
    expect(tabsFor("pro").map(([id]) => id)).not.toContain("jaimini");
  });

  it("leaves Lite exactly as it was", () => {
    vi.stubEnv("VITE_PRO", "off");
    expect(tabsFor("lite").map(([, label]) => label)).toEqual(LITE);
  });
});
