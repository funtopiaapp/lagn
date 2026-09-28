# Phase 5 design: a web app ready to become the iOS and Android apps

Status: built and QA-passed. Revision 2 (2026-09-21): CORS scoped to `/api` only; the service worker matches precached assets with `ignoreVary` (QA-P5-1).

## 1. Goal

One web app that is:

1. **Installable today** as a Progressive Web App (PWA) on Android, iOS and
   desktop, from the browser.
2. **Wrappable later** into store apps without a rewrite, either by
   Capacitor (a native shell around the same web code) or by PWA packagers
   (Bubblewrap/TWA for Android, PWABuilder for iOS).

The native apps can't be built in this environment: iOS needs Xcode, which
needs the owner's Apple ID, and Android needs SDK licence acceptance.
Everything that determines whether the conversion *works* can be built and
verified now, and that is this phase.

## 2. Architecture decision: thin client over the verified server

| Option | Verdict |
|---|---|
| **The app calls the verified server** | **Chosen** |
| Engine compiled into each phone app | Deferred |

Phase 4 showed every new maths platform (iOS, Android) needs a full validation
run. The Linux run found a release blocker on its first attempt. A thin client
ships the kernel already validated on macOS, Linux arm64 and Linux x86-64,
with measured cross-platform agreement. The trade-off: computing needs a
connection. The app shell opens offline and says plainly that a chart needs
one; it never shows stale or invented results.

## 3. What a wrapped app changes, and how the web app must be ready

| In a native wrapper | Required now |
|---|---|
| The app is served from `capacitor://localhost` or `https://localhost`, not the API's origin | **API base URL set at build time** (`VITE_API_BASE`, default same-origin). The server gets an **explicit CORS allow-list** for `/api` only, default empty. The web app's own files never carry CORS or `Vary: origin` headers. `X-Review-Token` is allowed only for listed origins. |
| No browser address bar; Android has a hardware back button | **History-based navigation**: form, chart, marriage and match are history entries, so Back moves within the app instead of quitting it |
| Notches and rounded screens | `viewport-fit=cover` and `env(safe-area-inset-*)` padding |
| iOS zooms into inputs below 16px | Every input at least 16px |
| Touch, not a mouse | Touch targets at least 44 x 44 px on phone widths; nothing hover-only |
| Launch without network | Service worker caches the app shell; the API is **never cached**; a clear offline banner |
| Store listings, home screen | Web app manifest (name, icons 192/512, maskable, theme, standalone), Apple touch icon and meta tags |

## 4. Service worker rules

- **Precache the shell:** `index.html` plus the build's hashed assets, from a
  list generated at build time. The cache name carries a content hash of that
  list.
- **Navigation:** network first, falling back to the cached shell.
- **Hashed assets:** cache first. Their names change whenever their content does.
- **`/api/*`: never intercepted or cached.** A chart must always come from the
  engine, never from a stale cache.
- **Updates:** a new worker takes over on the next launch, never mid-session,
  so one session never mixes two versions of the app.
- Old caches are deleted on activation.
- Precached assets are matched with `ignoreVary`. They are content-hashed and
  immutable, so request headers such as `Origin` must not cause a miss.

## 5. Security headers (server)

The SPA gets `Content-Security-Policy: default-src 'self'` (scripts, styles,
workers and manifest from self; `connect-src` self plus configured API
origins; `frame-ancestors 'none'`), `X-Content-Type-Options: nosniff` and
`Referrer-Policy: no-referrer`. A wrapped app's WebView honours the same
policy.

## 6. Conversion kit

- `capacitor.config.json`: app id `app.lagn`, name "Lagn", `webDir: dist`.
- `docs/phase5/CONVERSION.md`: the exact steps for the owner once Xcode and
  the Android SDK are available, and what each store will ask for.

## 7. Acceptance criteria (the QA gate)

1. **Installability:** Chromium reports no installability errors
   (`Page.getInstallabilityErrors`), the manifest is valid, and every icon
   exists at its declared size.
2. **Service worker:** it registers and controls the page; the app shell
   reloads offline; `/api/*` requests are never answered from cache (checked
   by request interception); the offline banner shows; computing offline gives
   a clear error, not a result.
3. **Cross-origin, the wrapped-app simulation:** the app served from one
   origin, built with `VITE_API_BASE` pointing at the server on another,
   works end to end with CORS allowing that origin, and fails for an origin
   not on the list.
4. **CORS unit tests:** allowed, disallowed, preflight, and the review-token
   header only for allowed origins.
5. **Mobile ergonomics at phone width:** input font sizes at least 16px, touch
   targets at least 44px, viewport meta correct, safe-area padding present.
6. **Back navigation** moves within the app, form to chart to tabs and back.
7. **CSP:** the app runs with the policy and no violations.
8. **No regressions:** `make qa` green, including all earlier E2E tests.
