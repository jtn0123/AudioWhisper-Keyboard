"""Fresh assertion-only mutations, restored byte-for-byte. Run with python3 -I."""
import json
from pathlib import Path
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
RUST = ROOT / 'lib/rust/crates'

def run(output, label, task, pattern):
    report_dir = ROOT / ('app/build/test-results/testDebugUnitTest' if task.startswith(':app:') else 'lib/kotlin/build/test-results/test')
    for report in report_dir.glob('TEST-*.xml'):
        report.unlink()
    started = time.time()
    with (output / (label + '.log')).open('wb') as log:
        result = subprocess.run(['./gradlew', '--stacktrace', task, '--tests', pattern], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
    failures = []
    tests = 0
    for report in report_dir.glob('TEST-*.xml'):
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
        ('restore-version', 'backup/src/lib.rs', 'version < 64', 'version < 65', ':app:testDebugUnitTest', '*Backup*MatchTest'),
        ('restore-conflict', 'backup/src/lib.rs', '.filter(|(_, v)| !bit(v))', '.filter(|(_, v)| bit(v))', ':app:testDebugUnitTest', '*Backup*MatchTest'),
        ('mime-wildcard', 'utilities/src/mime.rs', 'text(r"[^\\s]+")', 'text(r"[^\\s]*")', ':lib:kotlin:test', '*MimePolicyMatchTest'),
        ('ref-normalization', 'utilities/src/reference.rs', 'path[0] != 47', 'path[0] == 47', ':app:testDebugUnitTest', '*UtilityRulesMatchTest'),
    ]
    results = []
    for label, filename, old, new, task, pattern in edits:
        path = RUST / filename
        original = path.read_bytes()
        assert original.decode().count(old) == 1, 'Mutation location changed'
        try:
            path.write_text(original.decode().replace(old, new))
            result = run(output, label, task, pattern)
            result.update(mutation=label, restored=False)
            results.append(result)
            assert result['gradle_exit'] != 0 and result['assertion_failures'] > 0, 'Mutation did not fail a reference assertion'
        finally:
            path.write_bytes(original)
            if results and results[-1]['mutation'] == label:
                results[-1]['restored'] = path.read_bytes() == original
                (output / 'mutations.json').write_text(json.dumps(results, indent=2) + '\n')
        print(label + ': caught and restored', flush=True)
    for task, pattern in [(':app:testDebugUnitTest', '*Backup*MatchTest'), (':app:testDebugUnitTest', '*UtilityRulesMatchTest'), (':lib:kotlin:test', '*MimePolicyMatchTest')]:
        label = 'restored-' + pattern.strip('*')
        result = run(output, label, task, pattern)
        assert result['gradle_exit'] == 0 and result['tests'] > 0, 'Restored suite failed'
    print('All four mutations restored; match suites passed', flush=True)

if __name__ == '__main__':
    main()
