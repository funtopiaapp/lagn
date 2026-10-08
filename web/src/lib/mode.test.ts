// The Lite/Pro boundary. Specification: docs/phase13/DESIGN.md section 2.
//
// Three of that section's rules are testable here and are pinned below:
// Lite is the default, the choice survives a reload, and a storage failure
// must not take the app down - it falls back to Lite, which is the safe side
// for a surface the general public lands on.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { loadMode, saveMode } from "./mode";

beforeEach(() => localStorage.clear());
afterEach(() => vi.restoreAllMocks());

describe("which surface a visitor gets", () => {
  it("is Lite for someone who has never chosen", () => {
    expect(loadMode()).toBe("lite");
  });

  it("remembers Pro once it is chosen", () => {
    saveMode("pro");
    expect(loadMode()).toBe("pro");
  });

  it("forgets Pro when the reader goes back to Lite", () => {
    saveMode("pro");
    saveMode("lite");
    expect(loadMode()).toBe("lite");
    // Stored as an absence rather than the string "lite", so a future default
    // change reaches readers who never expressed a preference.
    expect(localStorage.getItem("lagn.mode")).toBeNull();
  });

  it("treats anything it does not recognise as Lite", () => {
    localStorage.setItem("lagn.mode", "expert");
    expect(loadMode()).toBe("lite");
  });
});

describe("when storage is unavailable", () => {
  it("still answers Lite rather than throwing", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("blocked"); });
    expect(loadMode()).toBe("lite");
  });

  it("still accepts a choice, for this page view", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("blocked"); });
    expect(() => saveMode("pro")).not.toThrow();
  });
});
