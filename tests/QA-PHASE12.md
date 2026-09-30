# QA report: Phase 12, the engine in the browser

Date: 2026-09-29. Spec: `docs/phase12/DESIGN.md`. Scope: compiling the engine
to WebAssembly, running it in the browser with no server, and publishing to
GitHub Pages.

## Verdict

**Pass.** Every reading is computed in the browser and agrees with the server
on everything discrete. 44 browser tests, 85 web unit tests; `make qa` and
Linux as recorded below.

## Acceptance criteria

| # | Criterion | Result |
|---|---|---|
| 1 | The engine compiles to WebAssembly and runs | pass - 2.1 MB module; `node scripts/wasm-smoke.mjs` loads it in 70 ms and produces all ten readings in 6 ms |
| 2 | Agreement with the server, in a real browser | pass - **0 discrete differences**; largest numeric difference 2.84e-14 degrees, limit 1e-9 |
| 3 | A reading happens with no network request | pass - the test records every `/api/` request during a chart, a reading and a place search, and asserts none |
| 4 | The HTTP transport still works | pass - those tests withhold the engine explicitly, so both paths stay covered |
| 5 | Root and path-prefix builds | pass - verified at `/` and `/lagn/`: assets, service worker scope, `start_url` and icons all follow |
| 6 | Earlier gates green | see below |

## What the browser downloads

8.6 MB once, then cached by the service worker: 2.1 MB engine, 1.9 MB
ephemeris, 4.2 MB place index, 352 KB corpus. The ephemeris is trimmed to the
1800-2400 block, which covers every living person; carrying all three blocks
would add 3.8 MB to every first visit.

## Defects found during QA

1. **CSP blocked WebAssembly.** `script-src 'self'` forbids compiling a
   module. Fixed with `'wasm-unsafe-eval'`, which permits WebAssembly and
   nothing else; a test now asserts `'unsafe-eval'` is still absent.
2. **A reading in the first second went to the server.** The app rendered
   before the engine finished loading, so `api.version()` fell through to
   HTTP - which on a static deployment would simply fail. Calls now await the
   transport decision; three tests cover it, including the case where the
   engine fails to load.
3. **`lagn_init` died with "allocation of 48 bytes failed".** Not an allocator
   fault: the timezone library, pointed at a zoneinfo directory, walks away
   under WASI. Using the copy compiled into the module fixes it and removes
   1.4 MB from the download.
4. **A 1 MB stack overflowed into the heap** inside Swiss Ephemeris, appearing
   as an unrelated allocation failure much later. Now 8 MB.
5. **22 browser tests broke** the moment the engine was deployed beside the
   app - they assert on API traffic that no longer happens. That is the
   feature working; the tests now withhold the engine and the two new ones opt
   in. Worth stating plainly: had those tests been "fixed" by loosening their
   assertions, the HTTP transport would have quietly lost its coverage.
6. **The no-date-math guard fired on the WASI clock.** `clock_time_get` must
   answer with the wall clock. Exempted with a reason, and a second test now
   asserts that file uses `Date` for that and nothing else.

## Not verified here

The GitHub Pages workflow has never run: it needs the repository to be public
with Pages enabled, which is the owner's action. The build steps it performs
are the ones run locally, and the subpath build is tested, but the deployment
itself is unproven until the first push.
