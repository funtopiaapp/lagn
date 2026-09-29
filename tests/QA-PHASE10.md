# QA report: Phase 10, running on the device

Date: 2026-09-28. Spec: `docs/phase10/DESIGN.md`. Scope: making the engine
usable with no server — a C ABI, an Android JNI layer, a client transport
switch, and the build scripts for both platforms.

## What was built

| Piece | State |
|---|---|
| `lagn_server::api` | the operations lifted out of the HTTP handlers; handlers are now three-line wrappers. **One implementation, two transports** |
| `crates/lagn-ffi` | a C ABI: init, version, topics, chart, topic, periods, family, match, places, offset, string release |
| `src/android.rs` | `Java_app_lagn_LagnNative_*` behind the `jni` feature, so Kotlin needs no C shim |
| `include/lagn.h` | what Swift compiles against |
| `web/src/lib/native.ts`, `api.ts` | uses `window.Lagn` when present, `fetch` otherwise |
| `scripts/build-ios.sh`, `build-android.sh` | cross-compile and stage the ~12 MB of data |

Artefacts on this machine: `liblagn_ffi.dylib` 2.5 MB, `liblagn_ffi.a` 45 MB
unstripped (the dynamic library is the realistic guide to app size).

## Acceptance criteria

| # | Criterion | Result |
|---|---|---|
| 1 | The C ABI and the HTTP server give the same JSON | **pass** — the real axum router and the real C ABI run side by side over 12 random births: charts, all 11 topics with write-ups, sensitive periods, family, match, places and offsets. 150+ comparisons, identical. Only wall-clock fields ("current", "running_now", "as_of_utc") are excluded, because the two calls happen moments apart |
| 2 | Bad input, null pointers, uninitialised library, many threads | pass — every case returns `{"error": ...}`; 8 concurrent threads |
| 3 | A device build never produces unapproved content | pass — asking for review mode still returns production content, every result approved |
| 4 | The header declares exactly what the library exports | pass — a test parses both and compares; it fails on drift in either direction |
| 5 | The client uses the bridge when present, HTTP otherwise | pass — 6 tests: no network on a device, identical payloads, errors reported the same way, and a **half-implemented bridge falls back to HTTP** rather than breaking the app |
| 6 | `make qa` and Linux stay green | see the addendum |

## Defects found during QA

1. The porutham count over the C ABI is 9, not 10 — Vasya is rejected for want
   of a verified table, and a device build only serves approved content. My
   test expected the reviewer-mode count; the library was right.
2. The TypeScript production build caught an unchecked index in the new test
   that `vitest` alone did not.
3. Clippy rejected `assert_eq!(x > 100, true)` in a test. Fixed.

## What is not verified, and why

**Nothing here has been cross-compiled or run on a phone.** This machine has
neither Xcode nor the Android NDK, so `aarch64-apple-ios` and the Android ABIs
cannot be linked, and the build scripts stop with a clear message. What is
proven is that the library builds, that its JSON is identical to the server's,
that the JNI layer typechecks, and that the client uses it correctly. The
remaining work is the Capacitor plugin wrapper and packaging, which needs those
toolchains installed.

The licence position is unchanged and now closer to mattering: shipping Swiss
Ephemeris inside an app is distribution.
