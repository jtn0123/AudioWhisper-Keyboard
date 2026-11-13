#!/bin/bash
# Use the exact agent-device command returned by device_open. Both paths must target the same device.
set -euo pipefail
cd "$(dirname "$0")/../.."
agent="${1:?usage: run.sh AGENT_DEVICE_COMMAND compose-rust-or-hybrid}"
renderer="${2:?renderer required}"
case "$renderer" in compose|rust|hybrid) ;; *) echo "Expected compose, rust or hybrid" >&2; exit 1 ;; esac
: "${ANDROID_SERIAL:?Set ANDROID_SERIAL to the device_open emulator serial}"
output="${3:-$PWD/build/stage3a-evidence}"
mkdir -p "$output"
for stream in 1 2 3 5; do
    adb shell cmd media_session volume --stream "$stream" --set 0
done > "$output/host-muting.log" 2>&1
adb shell cmd media_session volume --stream 4 --set 1 >> "$output/host-muting.log" 2>&1
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
adb logcat -d -s aw-render:D aw-compose:D AndroidRuntime:E > "$output/$renderer-typing-timing.log"
adb shell dumpsys gfxinfo com.jtn0123.audiowhisper.keyboard reset > /dev/null
for _ in 1 2 3 4 5 6 7 8; do
    "$agent" longpress 275 1750 500 > /dev/null
done
adb shell dumpsys gfxinfo com.jtn0123.audiowhisper.keyboard framestats > "$output/$renderer-gfx-popup.txt"
adb shell run-as com.jtn0123.renderprobe cat files/digest.json > "$output/$renderer-popup-final-digest.json"
adb logcat -d -s aw-render:D aw-compose:D AndroidRuntime:E > "$output/$renderer-timing.log"
if grep -Eq 'FATAL|AndroidRuntime' "$output/$renderer-timing.log"; then
    echo "Runtime failure; inspect $output/$renderer-timing.log" >&2
    exit 1
fi
printf 'Recorded %s: 300 key presses and 8 popup holds.\n' "$renderer"
