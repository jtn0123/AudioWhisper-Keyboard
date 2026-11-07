#!/bin/bash
# Installs the debug build on the phone without losing anything set up on it.
#
#  1. Copies the phone's settings, themes and personal dictionary to the Mac first
#     (.device-backup/, gitignored; the copy before that is kept as settings.prev.tar).
#  2. Installs as an update (adb install -r), which keeps the app's data. It never
#     uninstalls: that is what erases settings and the speech model.
#  3. If the settings are missing afterwards (a fresh install, or data cleared), puts
#     the Mac copy back before the keyboard first starts.
#  4. Makes AudioWhisper Keyboard the default keyboard again if Android switched away
#     (a force-stop does that), and grants microphone access.
#
# Usage: scripts/install-dev.sh [adb-serial]      (build first: ./gradlew :app:assembleDebug)
set -euo pipefail
cd "$(dirname "$0")/.."
ADB="${ANDROID_HOME:-$HOME/Library/Android/sdk}/platform-tools/adb"
SERIAL="${1:-${ANDROID_SERIAL:-}}"
adb() { if [ -n "$SERIAL" ]; then "$ADB" -s "$SERIAL" "$@"; else "$ADB" "$@"; fi; }

PKG=com.jtn0123.audiowhisper.keyboard
IME="$PKG/dev.patrickgold.florisboard.FlorisImeService"
APK=app/build/outputs/apk/debug/app-debug.apk
PREFS=jetpref_datastore/florisboard-app-prefs.jetpref
MODEL=files/models/sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8/encoder.int8.onnx
BACKUP_DIR=.device-backup
SNAPSHOT="$BACKUP_DIR/settings.tar"

on_phone() { adb shell "run-as $PKG $*" 2>/dev/null | tr -d '\r'; }
has_settings() { adb shell "run-as $PKG test -s $PREFS" 2>/dev/null; }

snapshot() {
    if [[ "$(adb shell pm path "$PKG" 2>/dev/null)" != *package:* ]] || ! has_settings; then
        echo "No settings on the phone to copy yet."
        return
    fi
    local paths
    paths=$(on_phone "sh -c 'ls -d jetpref_datastore files/ime databases/floris_user_dictionary* 2>/dev/null; true'" | tr '\n' ' ')
    mkdir -p "$BACKUP_DIR"
    adb exec-out "run-as $PKG tar -cf - $paths" > "$SNAPSHOT.new"
    local listing
    listing=$(tar -tf "$SNAPSHOT.new" 2>/dev/null || true)
    if ! grep -qxF "$PREFS" <<<"$listing"; then
        rm -f "$SNAPSHOT.new"
        echo "Could not copy the settings off the phone; stopping before installing." >&2
        exit 1
    fi
    [ -f "$SNAPSHOT" ] && mv "$SNAPSHOT" "$BACKUP_DIR/settings.prev.tar"
    mv "$SNAPSHOT.new" "$SNAPSHOT"
    echo "Copied the phone's settings to $SNAPSHOT."
}

install_update() {
    if ! adb install -r "$APK"; then
        cat >&2 <<'EOF'
The update was refused, and this script will not uninstall to force it: that would
erase your settings and the speech model. INSTALL_FAILED_UPDATE_INCOMPATIBLE means this
build was signed with a different debug key than the one on the phone (a different Mac,
or a new ~/.android/debug.keystore); build with the original key instead.
EOF
        exit 1
    fi
}

restore_if_missing() {
    if has_settings; then return; fi
    if [ ! -f "$SNAPSHOT" ]; then
        echo "The phone has no settings and there is no copy on the Mac; starting fresh."
        return
    fi
    adb exec-in "run-as $PKG tar -xf -" < "$SNAPSHOT"
    if restored_in_full; then echo "Settings were missing; restored them from $SNAPSHOT."
    else echo "Could not restore the settings from $SNAPSHOT." >&2; exit 1; fi
}

# adb exec-in can return before the phone has finished writing, so wait (up to 10 s)
# until the settings file is as long as the copy it came from.
restored_in_full() {
    local want have
    want=$(tar -xOf "$SNAPSHOT" "$PREFS" | wc -c | tr -d ' ')
    for _ in $(seq 20); do
        have=$(on_phone "wc -c $PREFS" | awk '{print $1}')
        [ "$have" = "$want" ] && return 0
        sleep 0.5
    done
    return 1
}

make_default() {
    adb shell ime enable "$IME" >/dev/null
    if [ "$(adb shell settings get secure default_input_method | tr -d '\r')" != "$IME" ]; then
        adb shell ime set "$IME" >/dev/null
        echo "Made AudioWhisper Keyboard the default keyboard again."
    fi
    adb shell pm grant "$PKG" android.permission.RECORD_AUDIO
}

[ -f "$APK" ] || { echo "No $APK; build it first: ./gradlew :app:assembleDebug" >&2; exit 1; }
snapshot
install_update
restore_if_missing
make_default
if ! adb shell "run-as $PKG test -s $MODEL" 2>/dev/null; then
    echo "Warning: the speech model is not on the phone; dictation won't work until it is copied in." >&2
fi
echo "Installed. Default keyboard: $(adb shell settings get secure default_input_method | tr -d '\r')"
