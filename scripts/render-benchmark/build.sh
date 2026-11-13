#!/bin/bash
# Builds the disposable editor outside production sources. Install/UI control use the device session.
set -euo pipefail
cd "$(dirname "$0")/../.."
sdk="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
tools="$sdk/build-tools/36.0.0"
android="$sdk/platforms/android-36/android.jar"
output="$PWD/build/render-benchmark"
mkdir -p "$output/classes" "$output/dex"
javac --release 11 -classpath "$android" -d "$output/classes" scripts/render-benchmark/*.java
"$tools/aapt2" compile --dir scripts/render-benchmark/res -o "$output/resources.zip"
"$tools/aapt2" link "$output/resources.zip" --manifest scripts/render-benchmark/AndroidManifest.xml -I "$android" -o "$output/probe-unsigned.apk"
"$tools/d8" --min-api 36 --lib "$android" --output "$output/dex" "$output/classes/com/jtn0123/renderprobe/"*.class
(cd "$output/dex" && zip -q "$output/probe-unsigned.apk" classes.dex)
"$tools/zipalign" -f 4 "$output/probe-unsigned.apk" "$output/probe-aligned.apk"
"$tools/apksigner" sign --ks "$HOME/.android/debug.keystore" --ks-key-alias androiddebugkey --ks-pass pass:android --key-pass pass:android --out "$output/probe.apk" "$output/probe-aligned.apk"
printf '%s\n' "$output/probe.apk"
