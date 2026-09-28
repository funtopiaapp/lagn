// The reviewer token lives in sessionStorage only: it disappears when the tab
// closes, and it never reaches localStorage. Storage can be unavailable
// (private mode, blocked site data), so every access is guarded.

const KEY = "lagn.reviewToken";

export function loadToken(): string {
  try { return sessionStorage.getItem(KEY) ?? ""; } catch { return ""; }
}

export function saveToken(token: string): void {
  try {
    if (token) sessionStorage.setItem(KEY, token);
    else sessionStorage.removeItem(KEY);
  } catch { /* storage unavailable: token lasts only for this page view */ }
}
