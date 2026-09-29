# Shipping to TestFlight

What it takes to get lagn onto a phone, in order, with the parts only you can
do marked. Nothing here has been run on this machine: it has Command Line
Tools but no Xcode, so no iOS SDK and no way to build or archive.

## 0. Two blockers, before any build

### 0.1 The Swiss Ephemeris licence — decide this first

Swiss Ephemeris is **AGPL-3.0 or a paid commercial licence** from Astrodienst.
Putting a build on TestFlight is distribution, so the choice stops being
theoretical the moment you upload.

The AGPL route does not work on the App Store in practice. GPL-family licences
forbid adding restrictions on what recipients may do with the software, and the
App Store terms impose exactly such restrictions; apps have been removed over
this before. Even if you accepted that, AGPL would require you to offer the
complete corresponding source of the app — which links the ephemeris — to every
user.

**So: buy the commercial licence before uploading a build.** It is a one-off
purchase per product from Astrodienst (https://www.astro.com/swisseph/). This
is not legal advice; if the money matters, have a lawyer read the two licences
against Apple's terms. What is not in doubt is that shipping under AGPL to the
App Store is a known-bad path.

The rule corpus is a separate question: it is data, not linked code, and stays
yours either way.

### 0.2 The Apple accounts — only you can do these

1. **Apple Developer Program**, USD 99/year, at developer.apple.com/programs.
   Enrolment can take a day or two, and as an individual you will need ID; as a
   company you need a D-U-N-S number, which takes longer. Start early.
2. **Xcode**, from the Mac App Store (about 10 GB). Then:
   ```bash
   sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
   xcodebuild -runFirstLaunch
   ```
3. **An App ID** and an **app record** in App Store Connect, bundle identifier
   `app.lagn` (it matches `web/capacitor.config.json`; change both together if
   you want a different one).

## 1. Build the engine for iOS

```bash
bash scripts/build-ios.sh          # device and simulator
```

Produces `build/ios/lib/<target>/liblagn_ffi.a`, `build/ios/include/lagn.h`,
and `build/ios/data/` — the ~12 MB of ephemeris, corpus, places and timezone
data the app bundles.

Sizes measured here: 2.4 MB shared library, 6.1 MB for a statically linked C
program, so expect roughly **18 MB installed** once the data is included.

## 2. Create the iOS app shell

```bash
cd web
npm install @capacitor/core @capacitor/cli @capacitor/ios
npm run build
npx cap add ios
npx cap sync ios
```

This generates `web/ios/App/App.xcodeproj`. Then, once:

1. Copy `ios/LagnNative/LagnPlugin.swift` into the App target.
2. Add `build/ios/lib/aarch64-apple-ios/liblagn_ffi.a` to **Link Binary With
   Libraries**, and `build/ios/include` to **Header Search Paths**.
3. Create a bridging header containing `#import "lagn.h"` and set
   **Objective-C Bridging Header** to it.
4. Drag `build/ios/data` into the project **as a folder reference** (blue, not
   yellow), so it keeps its structure inside the bundle. The plugin copies it
   to Application Support on first launch and marks it excluded from backup.
5. Set the deployment target to iOS 14 or later.

Check it runs in the simulator before going further:

```bash
npx cap open ios      # then Run
```

You should see the app work with no network: aeroplane mode is a fair test,
and a good one, because it proves the readings are local.

## 3. Signing, archive, upload

In Xcode, Signing & Capabilities: select your team, let it manage signing
automatically. Then:

```bash
bash scripts/ios-archive.sh            # archive and export an .ipa
```

Upload with Xcode's Organizer, or:

```bash
xcrun altool --upload-app -f build/ios/export/App.ipa -t ios \
  --apiKey "$ASC_KEY_ID" --apiIssuer "$ASC_ISSUER_ID"
```

(An App Store Connect API key, created under Users and Access → Integrations,
avoids putting an Apple ID password in a script.)

## 4. TestFlight

1. The build appears in App Store Connect after processing, usually 5-30
   minutes.
2. **Export compliance**: the app uses no encryption of its own. Set
   `ITSAppUsesNonExemptEncryption` to `false` in `Info.plist` and the question
   stops being asked on every build.
3. **Internal testers** (up to 100 people on your team) can install
   immediately, with no review.
4. **External testers** (up to 10,000) need Beta App Review — usually a day.
   For this app expect questions about the astrology content; the disclaimers
   already in every reading are the answer, and they are worth pointing at in
   the review notes.

### What to write in App Privacy

The answers are unusually simple, and worth stating plainly because they are a
selling point:

- **Data collected: none.** No accounts, no analytics, no network calls.
- Birth details and family members stay on the device.
- The engine runs locally, which is why the app works in aeroplane mode.

If that ever stops being true, this section is the first thing to change.

## 5. What is verified, and what is not

Verified on this machine: the engine builds as a native library, its JSON is
identical to the server's over 150+ comparisons, the C ABI is callable from a
plain C program (`bash scripts/native-demo.sh`), and the web client uses the
bridge when it exists.

Not verified: anything requiring the iOS SDK — cross-compilation, the Swift
plugin compiling, the app running on a device or simulator, signing, archiving
and upload. The Swift in `ios/LagnNative/` has been written carefully but has
never been compiled. Expect to fix small things on the first build.
