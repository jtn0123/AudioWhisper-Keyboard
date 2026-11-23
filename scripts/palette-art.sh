#!/bin/bash
# Rust's Material You palette against materialkolor on ART (PaletteArtDiff, an androidTest): counts
# of exact schemes and of each differing role, written to <out dir>/palette-art.tsv. The JVM
# differential (PaletteMatchTest) compares with HotSpot's maths; this one with the device's.
#
# Installs a fixture build (-PpaletteFixture=true keeps the classes the test calls by name) over the
# debug build on the device; install the ordinary one again afterwards.
#
# Usage: scripts/palette-art.sh <adb serial> <out dir> [accents, default 1000]
set -euo pipefail
cd "$(dirname "$0")/.."
SERIAL="${1:?adb serial}"
OUT="${2:?directory for the counts}"
SEEDS="${3:-1000}"
ADB="${ANDROID_HOME:-$HOME/Library/Android/sdk}/platform-tools/adb"
./gradlew -PpaletteFixture=true assembleDebug :app:assembleDebugAndroidTest
scripts/install-dev.sh "$SERIAL"
ANDROID_SERIAL="$SERIAL" scripts/install-dev.sh --probe app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk
RESULT=$("$ADB" -s "$SERIAL" shell am instrument -w -e class com.jtn0123.audiowhisper.render.PaletteArtDiff \
    -e seeds "$SEEDS" com.jtn0123.audiowhisper.keyboard.test/androidx.test.runner.AndroidJUnitRunner)
if ! grep -q "^OK (1 test)" <<<"$RESULT"; then
    echo "$RESULT" >&2
    exit 1
fi
mkdir -p "$OUT"
"$ADB" -s "$SERIAL" exec-out run-as com.jtn0123.audiowhisper.keyboard cat cache/palette-art.tsv >"$OUT/palette-art.tsv"
cat "$OUT/palette-art.tsv"
