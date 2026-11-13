#!/bin/bash
# Numeric input adapter for run.sh when no UiAutomation session may own the device.
set -euo pipefail
: "${ANDROID_SERIAL:?Dedicated runtime serial required}"
[ "$ANDROID_SERIAL" != emulator-5554 ] || exit 1
kind="${1:?press or longpress}"; shift
x="${1:?x}"; y="${2:?y}"; shift 2
count=1; hold=35; interval=85
if [ "$kind" = longpress ]; then
    hold="${1:?hold milliseconds}"; shift
elif [ "$kind" != press ]; then
    echo 'Only press and longpress are supported' >&2; exit 1
fi
while [ "$#" -gt 0 ]; do
    case "$1" in
        --count) count="$2";;
        --hold-ms) hold="$2";;
        --interval-ms) interval="$2";;
        *) echo 'Unknown input option' >&2; exit 1;;
    esac
    shift 2
done
for value in "$x" "$y" "$count" "$hold" "$interval"; do
    [[ "$value" =~ ^[0-9]+$ ]] || exit 1
done
pause=$(python3 -I -c 'import sys; print(int(sys.argv[1]) / 1000)' "$interval")
# All interpolated values above are checked numeric. No text entry passes this boundary.
adb shell "i=0; while [ \$i -lt $count ]; do input swipe $x $y $x $y $hold; sleep $pause; i=\$((i+1)); done"
