#!/bin/bash
# Records lib/rust/crates/icons/tests/fixtures/golden.tsv and keys.tsv: what Android made of each
# icon the Rust `icons` crate holds, through the Kotlin `iconPath` the crate replaced
# (IconGoldenDump, an androidTest, holds it verbatim), and which icon each key token drew. A golden
# line is the icon's name, its point count, then FNV-1a-64 of the path text and of the flattened
# points' raw bits; the crate's golden tests hold it to both, and to every key's icon.
#
# Installs a fixture build (-PiconsFixture=true keeps the Compose icons R8 drops) over the debug
# build on the device; install the ordinary one again afterwards. The full text and points are left
# in <out dir> for reading a difference.
#
# Usage: scripts/icon-golden.sh <adb serial> <out dir>
set -euo pipefail
cd "$(dirname "$0")/.."
SERIAL="${1:?adb serial}"
OUT="${2:?directory for the full dump}"
ADB="${ANDROID_HOME:-$HOME/Library/Android/sdk}/platform-tools/adb"
./gradlew -PiconsFixture=true assembleDebug :app:assembleDebugAndroidTest
scripts/install-dev.sh "$SERIAL"
ANDROID_SERIAL="$SERIAL" scripts/install-dev.sh --probe app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk
RESULT=$("$ADB" -s "$SERIAL" shell am instrument -w -e class com.jtn0123.audiowhisper.icons.IconGoldenDump \
    com.jtn0123.audiowhisper.keyboard.test/androidx.test.runner.AndroidJUnitRunner)
if ! grep -q "^OK (1 test)" <<<"$RESULT"; then
    echo "$RESULT" >&2
    exit 1
fi
mkdir -p "$OUT"
for f in icon-golden.tsv icon-points.tsv icon-keys.tsv; do
    "$ADB" -s "$SERIAL" exec-out run-as com.jtn0123.audiowhisper.keyboard cat "cache/$f" >"$OUT/$f"
done
python3 -I - "$OUT" lib/rust/crates/icons/tests/fixtures/golden.tsv <<'EOF'
import sys

def fnv(text):
    h = 0xCBF29CE484222325
    for b in text.encode():
        h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"

out, golden = sys.argv[1:]
paths = [line.split("\t", 3) for line in open(f"{out}/icon-golden.tsv").read().splitlines()]
points = dict(line.split("\t", 1) for line in open(f"{out}/icon-points.tsv").read().splitlines())
with open(golden, "w") as f:
    f.write("# scripts/icon-golden.sh: name, points, FNV-1a-64 of iconPath's text, of the raw points\n")
    for name, count, text_hash, _ in paths:
        f.write(f"{name}\t{count}\t{text_hash}\t{fnv(points[name])}\n")
with open(golden.replace("golden.tsv", "keys.tsv"), "w") as f:
    f.write("# scripts/icon-golden.sh: each key token keyIconVector drew, and the icons it was\n")
    f.write(open(f"{out}/icon-keys.tsv").read())
print(f"{len(paths)} icons")
EOF
