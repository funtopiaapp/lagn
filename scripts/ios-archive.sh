#!/usr/bin/env bash
# Archive the iOS app and export an .ipa ready for TestFlight.
# Needs Xcode, a signing team, and the app shell from docs/phase11/APPLE.md.
#
#   TEAM_ID=XXXXXXXXXX bash scripts/ios-archive.sh
set -euo pipefail
cd "$(dirname "$0")/.."

command -v xcodebuild >/dev/null || { echo "Xcode is not installed." >&2; exit 1; }
xcrun --sdk iphoneos --show-sdk-path >/dev/null 2>&1 || {
  echo "No iOS SDK. Install Xcode, then: sudo xcode-select -s /Applications/Xcode.app/Contents/Developer" >&2
  exit 1
}
: "${TEAM_ID:?set TEAM_ID to your Apple Developer team identifier}"

PROJECT="web/ios/App/App.xcworkspace"
[ -d "$PROJECT" ] || { echo "No app shell at $PROJECT. See docs/phase11/APPLE.md section 2." >&2; exit 1; }

OUT=build/ios
mkdir -p "$OUT/export"

# The engine and its data must be current before the app is packaged.
echo "== engine"
bash scripts/build-ios.sh device
echo "== web assets"
(cd web && npm run build && npx cap sync ios)

cat > "$OUT/ExportOptions.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>method</key><string>app-store-connect</string>
  <key>teamID</key><string>${TEAM_ID}</string>
  <key>uploadSymbols</key><true/>
  <key>signingStyle</key><string>automatic</string>
</dict>
</plist>
PLIST

echo "== archiving"
xcodebuild -workspace "$PROJECT" -scheme App -configuration Release \
  -destination 'generic/platform=iOS' -archivePath "$OUT/App.xcarchive" \
  DEVELOPMENT_TEAM="$TEAM_ID" archive

echo "== exporting"
xcodebuild -exportArchive -archivePath "$OUT/App.xcarchive" \
  -exportOptionsPlist "$OUT/ExportOptions.plist" -exportPath "$OUT/export"

echo
echo "ready: $OUT/export/App.ipa"
echo "upload with Xcode's Organizer, or:"
echo "  xcrun altool --upload-app -f $OUT/export/App.ipa -t ios --apiKey \$ASC_KEY_ID --apiIssuer \$ASC_ISSUER_ID"
