// Light and dark, chosen by the user or followed from the system.
// Storage can be unavailable (private mode, blocked site data), so every
// access is guarded and the app still works for this page view.

export type Theme = "system" | "light" | "dark";
export const THEMES: [Theme, string][] = [["system", "System"], ["light", "Light"], ["dark", "Dark"]];
const KEY = "lagn.theme";

export function loadTheme(): Theme {
  try {
    const v = localStorage.getItem(KEY);
    return v === "light" || v === "dark" || v === "system" ? v : "system";
  } catch { return "system"; }
}

export function saveTheme(t: Theme): void {
  try {
    if (t === "system") localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, t);
  } catch { /* not stored: the choice lasts for this page view */ }
}

export function systemPrefersDark(): boolean {
  try { return window.matchMedia("(prefers-color-scheme: dark)").matches; } catch { return false; }
}

/** The theme actually in force, with "system" resolved. */
export function resolveTheme(t: Theme): "light" | "dark" {
  return t === "system" ? (systemPrefersDark() ? "dark" : "light") : t;
}

/**
 * Put the choice on <html> so the stylesheet can act on it, and keep the
 * browser and wrapped-app chrome in step through <meta name="theme-color">.
 */
export function applyTheme(t: Theme): void {
  const root = document.documentElement;
  if (t === "system") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", t);
  const colour = resolveTheme(t) === "dark" ? "#101216" : "#ffffff";
  let meta = document.querySelector('meta[name="theme-color"]');
  if (!meta) {
    meta = document.createElement("meta");
    meta.setAttribute("name", "theme-color");
    document.head.appendChild(meta);
  }
  meta.setAttribute("content", colour);
}

/** Follow later OS changes while the setting is "system". Returns an unsubscribe. */
export function watchSystem(onChange: () => void): () => void {
  try {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  } catch { return () => {}; }
}
