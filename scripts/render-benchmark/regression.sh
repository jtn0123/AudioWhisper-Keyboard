#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/../.."
: "${ANDROID_SERIAL:?Dedicated runtime serial required}"
output="${1:?Fresh output directory required}"
mkdir -p "$output"
for renderer in compose rust; do
    python3 -I - "$renderer" "$output" <<'PY'
from pathlib import Path
import runpy
import sys
api = runpy.run_path('scripts/render-benchmark/a11y_device.py')
d = api['Device'](Path(sys.argv[2]) / sys.argv[1])
original = d.original_accessibility()
try:
    d.renderer(sys.argv[1])
    d.services(api['SERVICE'])
    d.probe(no_suggestions=True)
    tree = d.capture('preflight', lambda ns: any(n.get('text') in ('e', 'E') for n in ns))
    if any(n.get('text') == 'E' for n in tree['nodes']):
        d.tap(70, 2060)
    d.capture('lowercase', lambda ns: any(n.get('text') == 'e' for n in ns))
    d.reset()
finally:
    d.restore_accessibility(original)
PY
    scripts/render-benchmark/run.sh "${AGENT_DEVICE:-adb}" "$renderer" "$output/$renderer"
done
