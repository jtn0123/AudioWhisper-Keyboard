#!/bin/bash
# Copies the built-in neural voice (Pocket TTS int8, about 190 MB, and the "Loona" reference
# recording) onto the phone, where read-aloud uses it in place of the phone's own voice.
#
# Get the model from sherpa-onnx's releases (tts-models, sherpa-onnx-pocket-tts-int8-2026-01-26)
# and unpack it; pass the unpacked folder. It goes in the app's private files with run-as, so the
# debug build must be on the phone while this runs (the personal build installs over it and keeps
# the voice, as with the dictation model: see install-dev.sh).
#
# Usage: scripts/install-voice.sh <unpacked-model-folder> [adb-serial]
set -euo pipefail
cd "$(dirname "$0")/.."
ADB="${ANDROID_HOME:-$HOME/Library/Android/sdk}/platform-tools/adb"
SOURCE="${1:?Usage: scripts/install-voice.sh <unpacked-model-folder> [adb-serial]}"
SERIAL="${2:-${ANDROID_SERIAL:-}}"
adb() { if [ -n "$SERIAL" ]; then "$ADB" -s "$SERIAL" "$@"; else "$ADB" "$@"; fi; }

PKG=com.jtn0123.audiowhisper.keyboard
NAME=sherpa-onnx-pocket-tts-int8-2026-01-26
DEST=files/models/$NAME
STAGE=/data/local/tmp/$NAME
FILES=(lm_flow.int8.onnx lm_main.int8.onnx encoder.onnx decoder.int8.onnx text_conditioner.onnx
       vocab.json token_scores.json)
REFERENCE=test_wavs/loona.wav

for file in "${FILES[@]}" "$REFERENCE"; do
    [ -s "$SOURCE/$file" ] || { echo "Missing $SOURCE/$file; is that the unpacked $NAME folder?" >&2; exit 1; }
done
adb shell "run-as $PKG true" >/dev/null 2>&1 || {
    echo "The build on the phone isn't debuggable; install the debug build first (install-dev.sh)." >&2
    exit 1
}

adb shell "mkdir -p $STAGE"
for file in "${FILES[@]}" "$REFERENCE"; do adb push "$SOURCE/$file" "$STAGE/" >/dev/null; done
adb shell "run-as $PKG mkdir -p $DEST"
for file in "${FILES[@]}" "$REFERENCE"; do
    base=$(basename "$file")
    adb shell "run-as $PKG cp $STAGE/$base $DEST/$base"
    want=$(wc -c < "$SOURCE/$file" | tr -d ' ')
    have=$(adb shell "run-as $PKG wc -c $DEST/$base" | awk '{print $1}')
    [ "$want" = "$have" ] || { echo "Copying $base went wrong ($have of $want bytes)." >&2; exit 1; }
done
adb shell "rm -rf $STAGE"
echo "Installed the voice. Read-aloud uses it from the next reading."
