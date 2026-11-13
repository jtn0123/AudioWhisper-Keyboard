#!/bin/bash
# Restores this experiment's exact accessibility values and final development defaults.
set -euo pipefail
cd "$(dirname "$0")/../.."
: "${ANDROID_SERIAL:?Dedicated runtime serial required}"
python3 -I - "${1:?Output directory}" "${2:?Recorded original-accessibility.json}" <<'PY'
import json
from pathlib import Path
import runpy
import subprocess
import sys
api = runpy.run_path('scripts/render-benchmark/a11y_device.py')
d = api['Device'](sys.argv[1])
original = json.loads(Path(sys.argv[2]).read_text())
try:
    d.renderer('compose')
    prefs = d.prefs()
    keys = ('keyboard__rust_renderer_prototype', 'theme__mode', 'theme__day_theme_id')
    restored = {k: prefs.get(k) for k in keys}
    assert restored == {'keyboard__rust_renderer_prototype': 'false', 'theme__mode': '"ALWAYS_DAY"',
                        'theme__day_theme_id': '"org.florisboard.themes:floris_day"'}, restored
    d.save('preferences.json', restored)
finally:
    d.restore_accessibility(original)
for tag in ('aw-a11y', 'aw-render', 'aw-profile', 'aw-compose', 'aw-hybrid', 'aw-emoji'):
    d.shell('setprop', 'log.tag.' + tag, 'INFO')
d.shell('cmd', 'notification', 'set_dnd', 'off')
assert d.shell('settings', 'get', 'global', 'zen_mode').strip() == '0'
d.mute()
ime = api['PKG'] + '/dev.patrickgold.florisboard.FlorisImeService'
d.shell('ime', 'set', ime)
assert d.shell('settings', 'get', 'secure', 'default_input_method').strip() == ime
d.save('audio-streams.json', {str(stream): d.shell('cmd', 'media_session', 'volume', '--stream', stream, '--get')
                            for stream in (1, 2, 3, 4, 5, 10)})
d.wait(lambda: 'Bound services:{}' in d.shell('dumpsys', 'accessibility'), 'A service remains bound')
d.save('accessibility.json', d.original_accessibility())
(d.output / 'accessibility.txt').write_text(d.shell('dumpsys', 'accessibility'))
errors = [line for line in d.adb('logcat', '-d').splitlines() if 'FATAL' in line or 'AndroidRuntime' in line]
(d.output / 'runtime-errors.txt').write_text('\n'.join(errors) + ('\n' if errors else ''))
print(d.adb('uninstall', api['PROBE']).strip())
remaining = subprocess.run(['adb', '-s', d.serial, 'shell', 'pm', 'path', api['PROBE']], capture_output=True, text=True)
assert 'package:' not in remaining.stdout, 'Disposable probe is still installed'
d.save('cleanup.json', {'prototype': False, 'theme': 'Floris Day', 'mode': 'ALWAYS_DAY',
                        'accessibility': d.original_accessibility(), 'probe_installed': False,
                        'dnd': False, 'runtime_error_lines': len(errors)})
if errors:
    raise SystemExit('Inspect runtime-errors.txt')
PY
