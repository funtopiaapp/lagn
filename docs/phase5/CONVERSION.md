# Converting the web app into iOS and Android apps

The web app in `web/` is built to be wrapped without changes to its code
(see `DESIGN.md`). Two routes. Both are thin clients over the verified
server, so the numbers stay those of the validated engine.

## What only you can provide

| Need | Why |
|---|---|
| **A public HTTPS server** running `lagn-server` (the `Dockerfile` image) | The apps call it; stores reject apps that point at `http://` |
| **Xcode** (Mac App Store, signed in with your Apple ID) | Builds and signs iOS apps; not installable non-interactively |
| **Apple Developer Program** membership | Required to publish on the App Store |
| **Android SDK** (Android Studio, or command-line tools, accepting Google's licence) | Builds Android apps |
| **Google Play Console** account | Required to publish on Google Play |

## Route A: Capacitor (recommended; one codebase, both stores)

```bash
cd web
npm install --save-exact @capacitor/core @capacitor/cli @capacitor/ios @capacitor/android

# Point the app at your server, then build.
VITE_API_BASE=https://api.example.com npm run build

npx cap add ios          # needs Xcode
npx cap add android      # needs the Android SDK
npx cap sync
npx cap open ios         # build, sign, run in Xcode
npx cap open android     # build, sign, run in Android Studio
```

On the server, allow the apps' origins:

```bash
lagn-server ... --allow-origin capacitor://localhost --allow-origin https://localhost
```

`capacitor://localhost` is the iOS app's origin. `https://localhost` is the
Android app's, because `capacitor.config.json` sets `androidScheme: https`.

## Route B: package the installed PWA

- **Android:** Bubblewrap / Trusted Web Activity. Point it at the deployed
  site's `manifest.webmanifest`. Needs `/.well-known/assetlinks.json` served
  from the site, which Bubblewrap generates.
- **iOS:** PWABuilder's iOS package, which wraps the site in WKWebView.

## Before submitting to either store

1. Run `make release-check` against the deployed server: full QA plus tz
   data freshness.
2. Run `docker run --rm lagn-qa` on the production host itself.
3. Store reviewers look closely at astrology apps. Keep the in-app wording as
   shipped: it says "for guidance and study", names who reviewed the
   interpretations, and makes no medical, legal or financial claims.
