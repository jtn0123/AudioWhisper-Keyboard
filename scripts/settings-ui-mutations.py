"""Require fresh JVM assertion failures for each UI, editor and group 1 mutation, and fresh Rust test
panics for each layout and scroll mutation that only Rust tests see; always restore bytes."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
BASE = ROOT / 'lib/rust/crates'
CAT = BASE / 'render/src/settings_ui/catalogue'
SYNC_CLEAR, SYNC_SET = ('    (\n        "ONLY_{0}_EVENTS",\n        "enum__clipboard_sync_behavior__only_{1}_events",\n'
                         '        "enum__clipboard_sync_behavior__only_{1}_events__description",\n    ),\n'
                         .format(event, event.lower()) for event in ('CLEAR', 'SET'))
CASES = [
    ('toggle', BASE / 'render/src/settings_ui/events.rs',
     'self.write(r.key, (!self.yes(r.key)).to_string())', 'self.write(r.key, self.yes(r.key).to_string())'),
    ('default-stays-open', BASE / 'render/src/settings_ui/events.rs',
     'Action::Default => {\n                if let Some(m) = self.modal.take()',
     'Action::Default => {\n                if let Some(m) = self.modal.clone()'),
    ('surrogate-delete', BASE / 'render/src/settings_ui/editor.rs',
     'right += if pair { 2 } else { 1 };', 'right += if pair { 1 } else { 1 };'),
    ('predictive-cancel-commits', BASE / 'awnative/src/render_settings.rs',
     '2 => a.back_progress = 0.0,', '2 => a.back(),'),
    # Group 1: each must fail SettingsGroupOneMatchTest or SettingsResourcesTest.
    ('condition', CAT / 'clipboard.rs', '        &MINUTES,\n    )\n    .on(All(&[\n        HISTORY,\n'
     '        Yes("clipboard__history_auto_clean_old_enabled"),\n    ])),', '        &MINUTES,\n    )\n    .on(HISTORY),'),
    ('entry-order', CAT / 'clipboard.rs', SYNC_CLEAR + SYNC_SET, SYNC_SET + SYNC_CLEAR),
    ('no-preview', CAT / 'input_feedback.rs', '    hardware: 1,\n    preview: true,', '    hardware: 1,\n    preview: false,'),
    ('confirm-swap', CAT / 'media.rs', 'Kind::Confirm("pinned"', 'Kind::Confirm("recent"'),
    ('visible-fixed', CAT / 'smartbar.rs', '    .shown(SHARED),', '    .shown(Any(&[SHARED, EXTENDED])),'),
    ('stale-string-id', ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/settingsui/SettingsStringIds.kt',
     '    "action__yes" to R.string.action__yes,\n', ''),
    ('route-dropped', BASE / 'render/src/settings_ui/catalogue.rs', '    media::SCREEN,\n];', '];'),
    # The group 1 gallery's differences (stage4-followup4.md, part 1): each must fail a Rust test.
    ('no-stretch', BASE / 'render/src/settings_ui/overscroll.rs',
     '        let left = self.relax(delta);', '        let left = delta;'),
    ('fling-not-absorbed', BASE / 'render/src/settings_ui/overscroll.rs',
     'if velocity != 0.0 && self.stretch.edges[toward] > 0.0 {', 'if velocity != 0.0 && toward > 2 {'),
    ('one-line-switch-56', BASE / 'render/src/settings_ui/layout.rs',
     '                                64.0\n', '                                56.0\n'),
]
# The Rust test binary that must panic for each Rust-only case.
RUST = dict.fromkeys([
    'no-stretch',
    'fling-not-absorbed',
    'one-line-switch-56',
], 'settings_ui_overscroll')


def run_rust(output, name, target):
    with (output / (name + '.log')).open('wb') as log:
        result = subprocess.run(['cargo', 'test', '-q', '-p', 'render', '--test', target],
                                cwd=ROOT / 'lib/rust', stdout=log, stderr=subprocess.STDOUT)
    text = (output / (name + '.log')).read_text(errors='replace')
    failures = [{'suite': target, 'type': 'panic'} for line in text.splitlines() if 'panicked at' in line]
    return result.returncode, failures


def run(output, name):
    if name in RUST:
        return run_rust(output, name, RUST[name])
    started = time.time_ns()
    with (output / (name + '.log')).open('wb') as log:
        result = subprocess.run(['./gradlew', ':app:testDebugUnitTest', '--tests', '*SettingsEditorMatchTest',
                                 '--tests', '*SettingsUi*MatchTest', '--tests', '*SettingsGroupOneMatchTest',
                                 '--tests', '*SettingsResourcesTest'], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
    failures = []
    for path in (ROOT / 'app/build/test-results/testDebugUnitTest').glob('TEST-*settingsui.Settings*Test.xml'):
        if path.stat().st_mtime_ns < started:
            continue
        suite = ET.parse(path).getroot()
        for failure in suite.findall('.//failure'):
            if 'AssertionFailedError' in failure.attrib.get('type', ''):
                failures.append({'suite': suite.attrib['name'], 'type': failure.attrib['type']})
    return result.returncode, failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('cases', nargs='*', help='case names to run (default: all)')
    args = parser.parse_args()
    unknown = set(args.cases) - {name for name, _, _, _ in CASES}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    cases = [case for case in CASES if not args.cases or case[0] in args.cases]
    args.output.mkdir(parents=True, exist_ok=False)
    results = []
    originals = {path: path.read_bytes() for _, path, _, _ in cases}
    try:
        for name, path, old, new in cases:
            source = path.read_text()
            assert source.count(old) == 1, name
            path.write_text(source.replace(old, new))
            try:
                status, failures = run(args.output, name)
                assert status != 0 and failures, name + ' must fail fresh differential assertions'
                results.append({'mutation': name, 'exit': status, 'fresh_assertions': len(failures)})
            finally:
                path.write_bytes(originals[path])
            print(name + ': caught', flush=True)
    finally:
        for path, original in originals.items():
            path.write_bytes(original)
    restored, failures = run(args.output, 'restored')
    assert restored == 0 and not failures
    for target in sorted({RUST[name] for name, _, _, _ in cases if name in RUST}):
        restored, failures = run_rust(args.output, 'restored-' + target, target)
        assert restored == 0 and not failures, target
    report = {'mutations': results, 'restored_exit': restored,
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in originals}}
    (args.output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
