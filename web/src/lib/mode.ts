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
