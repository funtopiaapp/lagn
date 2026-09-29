# Phase 10 design: running on the device, with no server

Status: approved for build. The product owner asked on 2026-09-28 for the app
to run natively, without a backend.

## 1. Why this is possible

The server never held anything. `AppState` is three read-only things - the rule
corpus, the place index, the timezone database - plus an optional reviewer
token. No accounts, no database, no sessions; family readings already send both
birth records and store nothing. Every calculation is pure Rust over the
vendored Swiss Ephemeris, and the React app does no astrology or date
arithmetic at all. So the server was a delivery mechanism, not a dependency.

## 2. Shape

```
        React UI  (unchanged)
             │
      api.ts transport
        ┌────┴────┐
   window.Lagn   fetch()          ← whichever exists
        │            │
   C ABI (lagn-ffi)  HTTP (lagn-server)
        └────┬───────┘
      lagn_server::api            ← one implementation
             │
   lagn-rules → lagn-core → lagn-ephem → Swiss Ephemeris
```

`lagn_server::api` was extracted from the HTTP handlers in this phase. It knows
nothing of axum, sockets or headers. The handlers became three-line wrappers,
and the C ABI calls the same functions. **Neither transport can produce a
different reading, because there is only one implementation.**

## 3. The C ABI (`crates/lagn-ffi`)

JSON in, JSON out, in exactly the shapes the HTTP API uses.

| Function | Answers |
|---|---|
| `lagn_init(config_json)` | loads the reference data; call once |
| `lagn_ready()` | whether init succeeded |
| `lagn_version()`, `lagn_topics()` | engine and catalogue |
| `lagn_chart`, `lagn_topic`, `lagn_periods`, `lagn_family`, `lagn_match` | readings |
| `lagn_places`, `lagn_offset` | the birth form's lookups |
| `lagn_string_free(p)` | releases any returned string |

Contract: every call returns a NUL-terminated UTF-8 string the caller must free
with `lagn_string_free`; failures come back as `{"error": "..."}`, the same
shape HTTP uses, so the client handles one error path; every function is safe
from any thread, with Swiss Ephemeris serialised behind the lock already in
`lagn-ephem`; null and malformed input produce an error, never a crash.

**Reviewer mode does not exist on a device.** There is nowhere to keep a secret
on a phone, so `lagn_init` passes no token and the gate stays shut: a device
build can only ever produce approved content, even if a caller asks for review
mode. A test asserts it.

## 4. Data on the device

About 12 MB, copied out of the app bundle to app storage on first run, because
Android assets are not a real filesystem and Swiss Ephemeris opens files:

| | |
|---|---|
| `ephe/` | 5.8 MB (1200-2400 AD) |
| `data/places.tsv` | 4.2 MB |
| `data/zoneinfo` | 1.4 MB (optional: a copy is compiled in) |
| `corpus/` | 352 KB |

## 5. Platforms

- **iOS:** `cargo build --target aarch64-apple-ios --release` produces
  `liblagn_ffi.a`; Swift calls it directly through `include/lagn.h`. Needs
  Xcode.
- **Android:** the same code with the `jni` feature exposes
  `Java_app_lagn_LagnNative_*` so Kotlin can call it without a C shim. Needs
  the NDK, and `cargo-ndk` or a linker configured per ABI.
- Both are blocked on toolchains that must be installed by the owner. The
  library, the header, the JNI layer and the build scripts are written and
  compile on the host; only the cross-compilation and packaging remain.

## 6. Acceptance

1. **Parity:** the C ABI and the HTTP server return the same JSON for the same
   input, over many random births, across charts, every topic and its write-up,
   sensitive periods, family, match, places and offsets. Only wall-clock fields
   ("is this period running now") may differ.
2. Bad input, null pointers and an uninitialised library give errors, not
   crashes; many threads may call at once.
3. A device build never produces unapproved content.
4. The header declares exactly the functions the library exports.
5. The web client uses the native bridge when present and HTTP otherwise, with
   no change to the UI.
6. `make qa` and the Linux container stay green.
