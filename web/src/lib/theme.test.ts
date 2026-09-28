import { applyTheme, loadTheme, resolveTheme, saveTheme } from "./theme";

afterEach(() => { localStorage.clear(); document.documentElement.removeAttribute("data-theme"); vi.restoreAllMocks(); });

function system(dark: boolean) {
  vi.stubGlobal("matchMedia", vi.fn().mockReturnValue({ matches: dark, addEventListener: vi.fn(), removeEventListener: vi.fn() }));
}

describe("theme", () => {
  it("defaults to following the system and resolves it", () => {
    system(true);
    expect(loadTheme()).toBe("system");
    expect(resolveTheme("system")).toBe("dark");
    system(false);
    expect(resolveTheme("system")).toBe("light");
    // An explicit choice ignores the system.
    system(true);
    expect(resolveTheme("light")).toBe("light");
  });

  it("remembers a choice, and forgets it when set back to system", () => {
    saveTheme("dark");
    expect(loadTheme()).toBe("dark");
    saveTheme("light");
    expect(loadTheme()).toBe("light");
    saveTheme("system");
    expect(localStorage.getItem("lagn.theme")).toBeNull();
    expect(loadTheme()).toBe("system");
    localStorage.setItem("lagn.theme", "purple");
    expect(loadTheme()).toBe("system"); // anything unrecognised falls back
  });

  it("marks the document and keeps theme-color in step", () => {
    system(false);
    applyTheme("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
    expect(document.querySelector('meta[name="theme-color"]')!.getAttribute("content")).toBe("#101216");
    applyTheme("light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    expect(document.querySelector('meta[name="theme-color"]')!.getAttribute("content")).toBe("#ffffff");
    applyTheme("system");
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
  });

  it("survives storage being unavailable", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("blocked"); });
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("blocked"); });
    expect(() => saveTheme("dark")).not.toThrow();
    expect(loadTheme()).toBe("system");
  });
});
