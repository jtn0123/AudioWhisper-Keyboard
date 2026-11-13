#!/bin/bash
# Run only on the dedicated runtime serial returned by device_list.
set -euo pipefail
cd "$(dirname "$0")/../.."
: "${ANDROID_SERIAL:?Set ANDROID_SERIAL to the dedicated runtime serial}"
[ "$ANDROID_SERIAL" != emulator-5554 ] || { echo 'Reserved emulator' >&2; exit 1; }
exec python3 -I scripts/render-benchmark/a11y.py "$@"
