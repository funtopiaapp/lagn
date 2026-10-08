// Lite or Pro. Specification: docs/phase13/DESIGN.md section 2.
//
// Lite is the general public's surface: a reading, in plain words. Pro is the
// astrologer's: the apparatus, in its own vocabulary, including numbers whose
// variant has not been signed off yet.
//
// Four properties this has to keep, all of them from section 2:
//
//  - Lite is the default, so a first-time visitor never lands in Pro.
//  - the choice is device-local, stored like the theme, and survives a reload.
//  - it is *not* in the URL. A shared link must not drop a lay reader into
//    the professional surface, which is why this reads storage and nothing
//    else.
//  - it selects a presentation, never a computation. Both surfaces call the
//    same engine functions, so a number cannot differ between them.
//
// Storage can be unavailable (private mode, blocked site data), so every
// access is guarded and the app still works for this page view.

export type Mode = "lite" | "pro";

/** The choices, with the hint the toggle shows on hover and to assistive tech. */
export const MODES: [Mode, string, string][] = [
  ["lite", "Lite", "Readings in plain language. What most people want."],
  ["pro", "Pro", "Adds the astrologer's tools: Jaimini, Chara dasha and more. Some rest on variant choices no astrologer has signed off yet, each labelled where it appears."],
];

const KEY = "lagn.mode";

export function loadMode(): Mode {
  try {
    return localStorage.getItem(KEY) === "pro" ? "pro" : "lite";
  } catch {
    return "lite";
  }
}

export function saveMode(m: Mode): void {
  try {
    if (m === "lite") localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, m);
  } catch { /* not stored: the choice lasts for this page view */ }
}
