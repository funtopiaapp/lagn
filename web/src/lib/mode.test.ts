// The Lite/Pro boundary. Specification: docs/phase13/DESIGN.md section 2.
//
// Three of that section's rules are testable here and are pinned below:
// Lite is the default, the choice survives a reload, and a storage failure
// must not take the app down - it falls back to Lite, which is the safe side
// for a surface the general public lands on.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { loadMode, proEnabled, saveMode } from "./mode";

beforeEach(() => localStorage.clear());
afterEach(() => { vi.restoreAllMocks(); vi.unstubAllEnvs(); });

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

// The owner's kill switch.
//
// Set VITE_PRO=off at build time and the professional surface is simply not
// there. These pin the three things that could go wrong: a reader whose device
// remembers "pro" from before being shown a broken app, the switch being
// bypassed by writing to storage, and an unset variable accidentally removing
// features.

describe("the Pro kill switch", () => {
  it("is on when the variable is unset, so forgetting it never removes features", () => {
    vi.stubEnv("VITE_PRO", "");
    expect(proEnabled()).toBe(true);
  });

  it("accepts the several spellings an operator might reach for", () => {
    for (const off of ["off", "OFF", "0", "false", "no", " off "]) {
      vi.stubEnv("VITE_PRO", off);
      expect(proEnabled(), `${off} should disable Pro`).toBe(false);
    }
    for (const on of ["on", "1", "true", "yes", "anything"]) {
      vi.stubEnv("VITE_PRO", on);
      expect(proEnabled(), `${on} should leave Pro on`).toBe(true);
    }
  });

  it("puts a reader who had chosen Pro back on Lite rather than showing them nothing", () => {
    localStorage.setItem("lagn.mode", "pro");
    vi.stubEnv("VITE_PRO", "off");
    expect(loadMode()).toBe("lite");
  });

  it("cannot be bypassed by storing Pro while it is off", () => {
    vi.stubEnv("VITE_PRO", "off");
    saveMode("pro");
    expect(localStorage.getItem("lagn.mode")).toBeNull();
    expect(loadMode()).toBe("lite");
  });

  it("does not silently restore Pro for old readers when it is turned back on", () => {
    // Turning the surface off and on again should leave everyone on Lite
    // until they choose again - that is what "enable it explicitly" means at
    // the reader's end as well as the owner's.
    vi.stubEnv("VITE_PRO", "off");
    saveMode("pro");
    vi.stubEnv("VITE_PRO", "on");
    expect(loadMode()).toBe("lite");
  });
});
