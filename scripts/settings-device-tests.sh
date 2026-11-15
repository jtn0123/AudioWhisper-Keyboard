#!/bin/bash
# Open aw_keyboard_ssd through T3 first; export its exact AGENT_DEVICE command.
# Usage: scripts/settings-device-tests.sh emulator-5556 PRIVATE_BASELINE [ui|text|death|crash|all]
set -euo pipefail
cd "$(dirname "$0")/.."
serial="${1:?serial required}"
baseline="${2:-.device-backup/settings-device-$(date +%Y%m%d-%H%M%S)}"
mode="${3:-all}"
: "${AGENT_DEVICE:?Use the command returned by T3 device_open}"
[[ "$serial" != emulator-5554 ]]
[[ "$(adb -s "$serial" emu avd name | head -1 | tr -d '\r')" == aw_keyboard_ssd ]]
export ANDROID_SERIAL="$serial"
if [[ ! -f "$baseline/baseline.json" ]]; then
    python3 -I scripts/settings-device-baseline.py "$serial" "$baseline"
fi
trap 'python3 -I scripts/settings-device-baseline.py "$serial" "$baseline" --verify' EXIT
receipts="${SETTINGS_RECEIPTS:-build/stage4-followup3/instrumentation}"
export SETTINGS_RECEIPTS="$receipts"
mkdir -p "$receipts"
./gradlew -PsettingsFixture=true assembleDebug assembleDebugAndroidTest > "$receipts/build.log" 2>&1
scripts/install-dev.sh "$serial" > "$receipts/install.log" 2>&1
adb -s "$serial" install -r -t app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk
python3 -I scripts/settings-device-tests.py "$baseline" "$mode"
