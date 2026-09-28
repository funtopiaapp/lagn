# Phase 5 QA report: the convertible web app

Scope: PWA installability, the service worker, offline behaviour, CORS for
wrapped apps, security headers, mobile ergonomics and Back navigation.
Specification: `docs/phase5/DESIGN.md` revision 2. Date: 2026-09-21.
Verdict: **pass.** Also included: the AI corpus review, applied with the
product owner's permission.

## Acceptance criteria

| # | Criterion | Result |
|---|---|---|
| 1 | Installability | **Pass.** Chromium's `Page.getInstallabilityErrors` returns none. The manifest is valid, and every icon is exactly its declared size (192, 512, maskable 512, Apple 180). |
| 2 | Service worker and offline | **Pass after a fix (QA-P5-1).** The worker controls the page, the shell opens offline, the offline banner shows, and no `/api/` response is ever in the cache or answered offline. |
| 3 | Wrapped-app simulation | **Pass.** The app served from one origin and built with `VITE_API_BASE` computes charts through the API on another; an origin not on the list is refused by the browser. |
| 4 | CORS unit tests | **Pass.** Allowed and disallowed origins; the review-token header only in preflight for listed origins; cross-origin calls still need the review token; malformed `--allow-origin` values refused at startup. |
| 5 | Mobile ergonomics | **Pass after a fix (QA-P5-2).** Every input is at least 16px and every control at least 44px tall at phone width; `viewport-fit=cover`; safe-area padding. |
| 6 | Back navigation | **Pass.** Tab to tab to chart to form; Forward doesn't resurrect a chart from memory. |
| 7 | CSP | **Pass.** No violations across the full flow; header and meta policies both in force. |
| 8 | No regressions | **Pass.** 268 Rust tests in release and debug, 40 frontend unit tests, 24 browser tests, every oracle, `make qa` exits 0. Linux arm64 re-run: 536 tests and all oracles pass. |

## Findings

| ID | Severity | Finding | Fix |
|---|---|---|---|
| QA-P5-1 | **High** | **Blank screen offline whenever CORS is enabled**, the exact configuration the wrapped iOS and Android apps need. The CORS layer stamped the web app's own files with `Vary: origin`. The browser's module-script request carries an `Origin` header, the precache request didn't, so the worker's cache lookup missed and the JS never loaded. It first looked like a flaky test (failing only on a cold run). Diagnostics proved the HTML came from the worker while the bundle missed the cache, and `curl` confirmed the header. | CORS scoped to `/api` only; the worker matches immutable hashed assets with `ignoreVary`. Regression test asserts static files never vary by origin. Three consecutive cold runs pass. |
| QA-P5-2 | Medium | The reviewer sign-in field was 13.6px: it inherited the footer's small text, so iOS would zoom the page when it was tapped. | All form controls are at least 16px, wherever they sit. |
| QA-P4-9 | Low | The review sheet labelled every non-approved rule "Draft", so it reported 3 drafts that were actually rejected. | Each status counted separately; regression test. |
| - | Test | The ergonomics test read the viewport tag before loading the page. | Fixed. |

## The AI corpus review

Applied 2026-09-21 with the product owner's permission: 32 rules approved
(4 changed, 1 added), 3 rejected; 9 poruthams approved, Vasya rejected. Every
decision carries reviewer, date and reasoning (`docs/phase3/REVIEW-DECISIONS.md`).
The app shows who reviewed each result. The old wording claiming "a practising
astrologer" checks every rule was removed, and a test fails if it returns. The
rule-engine oracle re-verified the changed rules: 40,677 evaluations, 0
disagreements.

## Not verifiable here

Building the native iOS and Android apps. That needs Xcode (the owner's
Apple ID) and the Android SDK (licence acceptance). `docs/phase5/CONVERSION.md`
gives the exact steps.
