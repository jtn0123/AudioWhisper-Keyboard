#!/bin/bash
# Use the returned agent-device command, or "adb" for an isolated harness-only replay.
set -euo pipefail
cd "$(dirname "$0")/../.."
agent="${1:?usage: run.sh AGENT_DEVICE_COMMAND-or-adb compose-rust-or-hybrid}"
if [ "$agent" = adb ]; then agent="$PWD/scripts/render-benchmark/adb-input.sh"; fi
renderer="${2:?renderer required}"
case "$renderer" in compose|rust|hybrid) ;; *) echo "Expected compose, rust or hybrid" >&2; exit 1 ;; esac
: "${ANDROID_SERIAL:?Set ANDROID_SERIAL to the device_open emulator serial}"
output="${3:-$PWD/build/stage3a-evidence}"
mkdir -p "$output"
# Reject an unfocused/configuration-recreated editor before sending a trial.
adb shell dumpsys input_method | grep -q 'mInputShown=true'
adb shell run-as com.jtn0123.renderprobe cat files/digest.json > "$output/$renderer-initial-digest.json"
python3 -I - "$output/$renderer-initial-digest.json" <<'CHECK'
import json, sys
if json.load(open(sys.argv[1]))["utf16_units"] != 0:
    raise SystemExit("Clear the benchmark editor before running a trial")
CHECK
# `cmd media_session volume --set` reports success but changes nothing on the android-36 AVD;
# set through AudioService and stop unless the readback matches. Alarm (4) has a minimum of 1.
mute_stream() {
    adb shell cmd audio set-volume "$1" "$2" >> "$output/host-muting.log" 2>&1
    local level
    level=$(adb shell cmd audio get-stream-volume "$1" | tr -d '\r' | sed -n 's/.*-> *//p')
    echo "stream $1 level $level" >> "$output/host-muting.log"
    [ "$level" = "$2" ] || { echo "Stream $1 is at ${level:-unknown}, not $2" >&2; exit 1; }
}
: > "$output/host-muting.log"
for stream in 1 2 3 5; do mute_stream "$stream" 0; done
mute_stream 4 1
adb logcat -d -s AndroidRuntime:E > "$output/$renderer-before-runtime.log"
adb logcat -c
positions=("60 1750" "165 1750" "275 1750" "380 1750" "110 1900" "215 1900" "325 1900" "435 1900" "220 2060" "325 2060" "435 2060" "540 2060")
batch=0
for position in "${positions[@]}"; do
    adb shell dumpsys gfxinfo com.jtn0123.audiowhisper.keyboard reset > /dev/null
    read -r x y <<< "$position"
    "$agent" press "$x" "$y" --count 25 --hold-ms 35 --interval-ms 85 > "$output/$renderer-actions-$batch.txt"
    adb shell dumpsys gfxinfo com.jtn0123.audiowhisper.keyboard framestats > "$output/$renderer-gfx-$batch.txt"
    batch=$((batch + 1))
done
adb shell run-as com.jtn0123.renderprobe cat files/digest.json > "$output/$renderer-typing-digest.json"
adb logcat -d -s aw-render:D aw-compose:D aw-profile:D AndroidRuntime:E > "$output/$renderer-typing-timing.log"
adb shell dumpsys gfxinfo com.jtn0123.audiowhisper.keyboard reset > /dev/null
for _ in 1 2 3 4 5 6 7 8; do
    "$agent" longpress 275 1750 500 > /dev/null
done
adb shell dumpsys gfxinfo com.jtn0123.audiowhisper.keyboard framestats > "$output/$renderer-gfx-popup.txt"
adb shell run-as com.jtn0123.renderprobe cat files/digest.json > "$output/$renderer-popup-final-digest.json"
if [ "${4:-}" = glide ]; then
    adb shell dumpsys gfxinfo com.jtn0123.audiowhisper.keyboard reset > /dev/null
    python3 -I scripts/render-benchmark/gestures.py "$agent" "$output/glide" --glide-only
    adb shell dumpsys gfxinfo com.jtn0123.audiowhisper.keyboard framestats > "$output/$renderer-gfx-glide.txt"
fi
adb logcat -d -s aw-render:D aw-compose:D aw-profile:D AndroidRuntime:E > "$output/$renderer-timing.log"
if grep -Eq 'FATAL|AndroidRuntime|draw_error|post_error|hybrid_draw_error' "$output/$renderer-timing.log"; then
    echo "Runtime failure; inspect $output/$renderer-timing.log" >&2
    exit 1
fi
python3 -I - "$output/$renderer-typing-digest.json" "$output/$renderer-popup-final-digest.json" <<'CHECK'
import json, sys
expected = [(300, "809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3"),
            (308, "c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555")]
for path, (length, digest) in zip(sys.argv[1:], expected):
    actual = json.load(open(path))
    if {key: actual[key] for key in ("utf16_units", "sha256")} != {"utf16_units": length, "sha256": digest}:
        raise SystemExit("Benchmark digest mismatch: " + path)
CHECK
printf 'Recorded %s: 300 key presses and 8 popup holds.\n' "$renderer"
