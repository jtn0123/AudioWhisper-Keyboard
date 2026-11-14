"""Mutate actual production settings policy; require fresh assertions, restore exact bytes.
Run with python3 -I scripts/settings-mutations.py build/stage2-settings/mutations-final.
"""
import json
from pathlib import Path
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
RUST = ROOT / 'lib/rust/crates/settings/src'


def run(output, label, pattern):
    reports = ROOT / 'app/build/test-results/testDebugUnitTest'
    for report in reports.glob('TEST-*.xml'):
        report.unlink()
    started = time.time()
    with (output / (label + '.log')).open('wb') as log:
        result = subprocess.run(['./gradlew', '--stacktrace', ':app:testDebugUnitTest', '--tests', pattern],
                                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
    failures = []
    tests = 0
    for report in reports.glob('TEST-*.xml'):
        assert report.stat().st_mtime >= started - 1, 'Stale report'
        tree = ET.parse(report).getroot()
        tests += int(tree.attrib.get('tests', 0))
        failures += [f.attrib.get('type', '') for f in tree.iter('failure')]
    return {'gradle_exit': result.returncode, 'tests': tests,
            'assertion_failures': sum('AssertionFailedError' in f for f in failures)}


def main():
    output = ROOT / sys.argv[1]
    output.mkdir(parents=True, exist_ok=True)
    edits = [
        ('default', 'catalogue.tsv', 'keyboard__number_row\tb\tfalse', 'keyboard__number_row\tb\ttrue', '*SettingsDefaultsMatchTest'),
        ('clamp-bound', 'catalogue.tsv', '300\t100\t700\t10', '300\t101\t700\t10', '*SettingsDefaultsMatchTest'),
        ('migration', 'migration.rs', '"advanced__settings_theme" => target = "other__settings_theme"',
         '"advanced__settings_theme" => target = "other__incorrect_theme"', '*SettingsMigrationMatchTest'),
        ('active-fallback', 'subtype.rs', 'else { Some(0) }', 'else { Some(ids.len() - 1) }', '*SettingsSubtypeMatchTest'),
    ]
    results = []
    for label, filename, old, new, pattern in edits:
        path = RUST / filename
        original = path.read_bytes()
        assert original.decode().count(old) == 1, 'Mutation location changed: ' + label
        try:
            path.write_text(original.decode().replace(old, new))
            result = run(output, label, pattern)
            result.update(mutation=label, restored=False)
            results.append(result)
            assert result['gradle_exit'] != 0 and result['assertion_failures'] > 0, 'Not caught by assertions'
        finally:
            path.write_bytes(original)
            if results and results[-1]['mutation'] == label:
                results[-1]['restored'] = path.read_bytes() == original
                (output / 'mutations.json').write_text(json.dumps(results, indent=2) + '\n')
        print(label + ': caught and restored', flush=True)
    restored = run(output, 'restored', '*Settings*MatchTest')
    (output / 'restored.json').write_text(json.dumps(restored, indent=2) + '\n')
    assert restored['gradle_exit'] == 0 and restored['tests'] > 0, 'Restored suites failed'
    print('Four mutations restored; all settings match suites pass', flush=True)


if __name__ == '__main__':
    main()
