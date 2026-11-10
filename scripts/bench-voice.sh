#!/bin/bash
# Times the built-in neural voice on a phone or emulator, outside the app: the same sherpa-onnx
# library and model, one sentence at a time as read-aloud generates it (tools/voice-bench).
# Prints per-sentence timings, the overall speed (×1 is real time; read-aloud needs better than
# that), how long until the first word, and the longest silence a listener would hear.
#
# Needs no app on the device: everything goes in /data/local/tmp/voice-bench.
#
# SLOW=<share> runs an emulator at that share of its CPU time (SIGSTOP/SIGCONT on the emulator
# process), to stand in for a slower phone. 0.42 is about an S24 Ultra on an M5 Pro, from
# Geekbench 6 (2,300 vs 4,300 single-core, and the phone's 4 threads land on one X4 and three
# A720s): see tools/voice-bench/README.md.
#
# Usage: [SLOW=0.42] scripts/bench-voice.sh <unpacked-model-folder> [adb-serial] [threads] [flow-steps]
set -euo pipefail
cd "$(dirname "$0")/.."
SDK="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
ADB="$SDK/platform-tools/adb"
SOURCE="${1:?Usage: scripts/bench-voice.sh <unpacked-model-folder> [adb-serial] [threads] [flow-steps]}"
SERIAL="${2:-${ANDROID_SERIAL:-}}"
THREADS="${3:-4}"
STEPS="${4:-5}"
adb() { if [ -n "$SERIAL" ]; then "$ADB" -s "$SERIAL" "$@"; else "$ADB" "$@"; fi; }

AAR=app/libs/sherpa-onnx-2.0.1.aar
OUT=build/voice-bench
DEVICE=/data/local/tmp/voice-bench
FILES=(lm_flow.int8.onnx lm_main.int8.onnx encoder.onnx decoder.int8.onnx text_conditioner.onnx
       vocab.json token_scores.json test_wavs/loona.wav)

# Build: javac against the AAR's classes, then d8 into one dex with the Kotlin runtime they need.
STDLIB=$(find "$HOME/.gradle/caches/modules-2" -name 'kotlin-stdlib-2.*.jar' ! -name '*sources*' | sort -V | tail -1)
PLATFORM=$(find "$SDK/platforms" -name android.jar | sort -V | tail -1)
D8=$(find "$SDK/build-tools" -name d8 | sort -V | tail -1)
[ -n "$STDLIB" ] || { echo "No Kotlin stdlib in the Gradle cache; build the app once first." >&2; exit 1; }
rm -rf "$OUT" && mkdir -p "$OUT/classes" "$OUT/aar"
unzip -q -o "$AAR" classes.jar 'jni/arm64-v8a/*' -d "$OUT/aar"
javac -nowarn --release 17 -cp "$OUT/aar/classes.jar:$STDLIB:$PLATFORM" -d "$OUT/classes" tools/voice-bench/Bench.java
"$D8" --min-api 30 --lib "$PLATFORM" --output "$OUT" "$OUT"/classes/*.class "$OUT/aar/classes.jar" "$STDLIB"

adb shell "mkdir -p $DEVICE/lib $DEVICE/model"
adb push "$OUT/classes.dex" tools/voice-bench/text.txt "$DEVICE/" >/dev/null
adb push "$OUT"/aar/jni/arm64-v8a/*.so "$DEVICE/lib/" >/dev/null
for file in "${FILES[@]}"; do
    [ -s "$SOURCE/$file" ] || { echo "Missing $SOURCE/$file" >&2; exit 1; }
    adb push "$SOURCE/$file" "$DEVICE/model/" >/dev/null
done

THROTTLE=""
if [ -n "${SLOW:-}" ]; then
    PORT="${SERIAL#emulator-}"
    PID=$(pgrep -f "qemu-system.*-port $PORT" || pgrep -f "qemu-system" | head -1)
    [ -n "$PID" ] || { echo "SLOW is for emulators; no emulator process found." >&2; exit 1; }
    python3 -I tools/voice-bench/throttle.py "$PID" "$SLOW" &
    THROTTLE=$!
    trap 'kill $THROTTLE 2>/dev/null || true' EXIT
fi

adb shell "cd $DEVICE && LD_LIBRARY_PATH=lib CLASSPATH=classes.dex app_process / Bench model model/loona.wav $THREADS $STEPS text.txt" \
    2>&1 | grep -E '^(load|chars|TOTAL)' | tee "$OUT/result.txt"
python3 -I tools/voice-bench/gaps.py "$OUT/result.txt"
