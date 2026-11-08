#!/bin/sh
# Regenerates crates/chars/src/tables: jvm.rs from this computer's Java, and android.rs from the
# Android device or emulator named by ANDROID_SERIAL, by running tools/JavaChars.java on each.
# Use a JDK and an Android version with the same Unicode as the phones the keyboard runs on.
#
#     ANDROID_SERIAL=emulator-5554 tools/generate-char-tables.sh
#
# Needs java and javac, adb, and d8 from the Android SDK's build-tools (found through ANDROID_HOME
# or local.properties' sdk.dir).
set -eu
: "${ANDROID_SERIAL:?set ANDROID_SERIAL to the device or emulator to read the Android rules from}"
export ANDROID_SERIAL
cd "$(dirname "$0")/.."

sdk=${ANDROID_HOME:-}
if [ -z "$sdk" ]; then sdk=$(sed -n 's/^sdk.dir=//p' ../../local.properties); fi
d8=$(ls -d "$sdk"/build-tools/*/d8 | sort -V | tail -n 1)
tables=crates/chars/src/tables
work=build/javachars
rm -rf "$work"
mkdir -p "$work"

java tools/JavaChars.java > "$tables/jvm.rs"

javac --release 11 -d "$work" tools/JavaChars.java
"$d8" --min-api 26 --output "$work" "$work"/*.class
adb push "$work/classes.dex" /data/local/tmp/javachars.dex > /dev/null
adb shell CLASSPATH=/data/local/tmp/javachars.dex app_process /data/local/tmp JavaChars > "$tables/android.rs"
adb shell rm /data/local/tmp/javachars.dex

cargo fmt -p chars
