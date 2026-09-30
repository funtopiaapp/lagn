# Phase 12 design: the engine in the browser

Status: built. The product owner chose web-only distribution (AGPL), then
asked for free hosting with no credit card. That points at static hosting,
which means no server, which means the engine runs in the browser.

## 1. Shape

```
        React UI  (unchanged)
             │
      api.ts transport
     ┌───────┼───────┐
window.Lagn  │    fetch()
  ┌──┴──┐    │
native  WASM │            ← whichever exists, decided at start-up
       └─────┴── lagn_server::api → lagn-rules → lagn-core → Swiss Ephemeris
```

The C ABI from phase 10 is reused unchanged: the browser calls the same twelve
exports an iOS app would. Nothing in the UI knows which transport answered.

## 2. How the C runs at all

Swiss Ephemeris is C that opens `.se1` files, so the module is built for
`wasm32-wasip1`: wasi-sdk compiles the C, and rustc links it with its own lld
and wasi-libc. Passing the external linker as well makes it build a
position-independent shared library, which fails.

Three changes were needed in the engine:

- `-DNO_SWE_GLP` for wasm: upstream's `swe_get_library_path` calls `dladdr`,
  and WebAssembly has no dynamic loader. It is upstream's own switch, and
  nothing here calls that function.
- An 8 MB stack (`-zstack-size`): Swiss Ephemeris has large frames, and the
  1 MB default overflows into the heap, which shows up later as an
  inexplicable allocation failure.
- `lagn_buffer_alloc` / `lagn_buffer_free`: a WebAssembly host cannot place a
  string in the module's memory without asking the module for space.

## 3. The filesystem

The browser has none, so `src/lib/wasi.ts` provides one: a read-only
in-memory tree, implementing the 17 WASI calls the module actually imports.
It is a host for one known guest, not a general runtime.

The reference data is fetched once and cached by the service worker:

| | |
|---|---|
| `lagn.wasm` | 2.1 MB |
| `ephe/` (1800-2400 only) | 1.9 MB |
| `corpus/` | 352 KB |
| `data/places.tsv` | 4.2 MB |

**The ephemeris ships only the 1800-2400 block.** It covers every living
person and their transits to the end of a long life; including 1200-1800 and
2400-3000 would add 3.8 MB to every first visit.

**The timezone database is the copy compiled into the module** (2026c), not
the shipped 2026d. Pointing the timezone library at a zoneinfo *directory*
sends it into a runaway walk under WASI - found by running it - and the
built-in copy also saves 1.4 MB.

## 4. Deciding the transport

The UI renders immediately, but a reading made in the first moment must not
fall through to a server that does not exist. `lib/transport.ts` holds a
promise that `api.ts` awaits before choosing. When no decision is pending it
resolves at once, so nothing else is slowed down.

## 5. Content-Security-Policy

`script-src` gains `'wasm-unsafe-eval'`, which permits compiling WebAssembly
and nothing else. `'unsafe-eval'` stays forbidden, and a test asserts both.

## 6. What "agrees with the server" means here

wasi-libc's `sin` and `cos` differ from the host's in the last bits, so the
browser is not byte-identical to the server - no more than macOS is to Linux,
which `scripts/qa_cross_platform.py` has measured all along. The browser test
applies that same standard: **everything discrete must match exactly** - every
sign, house, verdict, score and sentence - and numeric drift is measured, with
a limit of 1e-9. Observed: 0 discrete differences, largest numeric difference
2.84e-14 degrees, about a ten-billionth of an arcsecond.

## 7. Deployment

`.github/workflows/pages.yml` builds the engine and the app and publishes to
GitHub Pages on a push to `master`. A project site is served from `/<repo>/`,
so the build takes a `BASE_URL`, and the service worker, the manifest's
`start_url` and the icons all follow it.

## 8. Acceptance

1. The engine compiles to WebAssembly and runs, with every reading available.
2. In a real browser: 0 discrete differences against the server, numeric drift
   below 1e-9.
3. A chart, a reading and a place search happen with **no** network request.
4. The app still works with no engine deployed (HTTP), and the tests that
   cover that transport withhold the engine explicitly.
5. The build works at the root and under a path prefix.
6. `make qa` and the Linux container stay green.
