"""Require fresh JVM assertion failures for each UI, editor, group 1, group 2, licences, Other, Theme, language, user
dictionary, project licence, theme manager, setup and theme editor mutation, and fresh Rust test panics for each mutation that only Rust tests see; always restore bytes."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

# The cases live in settings_ui_mutation_cases, one module per screen. -I leaves this folder off the path.
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
from settings_ui_mutation_cases import CASES, RUST  # noqa: E402
from settings_ui_mutation_cases.paths import ROOT  # noqa: E402

# Suites whose assertion failures count, besides the settingsui ones: the first screen a fresh activity opens.
SUITES = ('TEST-*settingsui.Settings*Test.xml', 'TEST-*florisboard.app.SettingsEntryTest.xml',
          'TEST-*florisboard.app.ComposeFallbackLinksTest.xml')


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
                                 '--tests', '*SettingsUi*MatchTest', '--tests', '*SettingsGroupOneMatchTest', '--tests', '*SettingsGroupTwoMatchTest',
                                 '--tests', '*SettingsLicencesMatchTest', '--tests', '*SettingsOtherThemeMatchTest',
                                 '--tests', '*SettingsResourcesTest', '--tests', '*SettingsLanguagesMatchTest',
                                 '--tests', '*SettingsWords*Test', '--tests', '*SettingsProjectLicenceMatchTest',
                                 '--tests', '*SettingsThemesMatchTest', '--tests', '*SettingsSetupMatchTest',
                                 '--tests', '*SettingsThemeEditor*MatchTest', '--tests', '*SettingsThemeCodeMatchTest',
                                 '--tests', '*SettingsKeyRecorderTest', '--tests', '*SettingsExtensionsMatchTest',
                                 '--tests', '*SettingsEntryTest', '--tests', '*ComposeFallbackLinksTest'], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
    failures = []
    xml = ROOT / 'app/build/test-results/testDebugUnitTest'
    for path in (path for pattern in SUITES for path in xml.glob(pattern)):
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
