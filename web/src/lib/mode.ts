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

/**
 * Whether the professional surface exists in this build at all.
 *
 * This is the owner's kill switch, and it is deliberately coarser than the
 * per-reader toggle. Set `VITE_PRO=off` at build time and Pro is simply not
 * there: no toggle in the header, no professional tabs, and a reader whose
 * device still remembers `pro` from before is put back on Lite rather than
 * shown an empty app.
 *
 * Default on. An unset variable keeps the surface exactly as it is today, so
 * forgetting to set it can never silently remove features; only setting it to
 * `off` does anything.
 *
 * Why build time rather than runtime: the app is a static site, so there is no
 * server to ask, and a runtime config file would be one more request on every
 * first load for a value that changes once a year. Flipping it is a one-line
 * change in the deploy workflow. `docs/INTEGRATION.md` says where.
 */
export function proEnabled(): boolean {
  // The variable is consulted before the folded literal, and the two can
  // never disagree in a real build because both are derived from VITE_PRO.
  // Reading it first is what lets the test runner exercise both states
  // without rebuilding, since `define` substitutes PRO_BUILD under vitest too
  // and a literal cannot be stubbed.
  const raw = import.meta.env.VITE_PRO;
  if (raw !== undefined && raw !== null && String(raw).trim() !== "") {
    const v = String(raw).trim().toLowerCase();
    return !(v === "off" || v === "0" || v === "false" || v === "no");
  }
  if (typeof PRO_BUILD === "boolean") return PRO_BUILD;
  return true;
}

/** The choices, with the hint the toggle shows on hover and to assistive tech. */
export const MODES: [Mode, string, string][] = [
  ["lite", "Lite", "Readings in plain language. What most people want."],
  ["pro", "Pro", "Adds the astrologer's tools: Jaimini, Chara dasha and more. Some rest on variant choices no astrologer has signed off yet, each labelled where it appears."],
];

const KEY = "lagn.mode";

export function loadMode(): Mode {
  // The kill switch wins over anything the device remembers. Someone who
  // chose Pro before it was turned off gets Lite, not a blank screen.
  if (!proEnabled()) return "lite";
  try {
    return localStorage.getItem(KEY) === "pro" ? "pro" : "lite";
  } catch {
    return "lite";
  }
}

export function saveMode(m: Mode): void {
  // Nothing can store "pro" while the surface is off, so turning it back on
  // later does not silently restore it for readers who had it before. They
  // choose again, which is what "explicitly enable" should mean on both ends.
  if (!proEnabled()) return;
  try {
    if (m === "lite") localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, m);
  } catch { /* not stored: the choice lasts for this page view */ }
}
