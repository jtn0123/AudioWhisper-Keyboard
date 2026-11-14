"""Compile a deliberately unexported production declaration; require the APK gate to reject it."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'lib/rust/src/main/kotlin/com/jtn0123/audiowhisper/rust/SuggesterNative.kt'


def run(output, phase):
    with (output / (phase + '.log')).open('wb') as log:
        return subprocess.run(['./gradlew', '--stacktrace', ':app:checkNativeDeclarations'],
                              cwd=ROOT, stdout=log, stderr=subprocess.STDOUT).returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    original = SOURCE.read_bytes()
    try:
        source = original.decode()
        at = source.rindex('}')
        SOURCE.write_text(source[:at] + '    @JvmStatic external fun mutationMissingExport(): Int\n' + source[at:])
        mutant = run(args.output, 'mutant')
        log = (args.output / 'mutant.log').read_text()
        assert mutant != 0
        assert 'Production native declarations missing from normal APK:' in log
        assert 'Java_com_jtn0123_audiowhisper_rust_SuggesterNative_mutationMissingExport' in log
        assert '> Task :app:checkNativeDeclarations FAILED' in log
    finally:
        SOURCE.write_bytes(original)
        assert SOURCE.read_bytes() == original
    restored = run(args.output, 'restored')
    report = {'mutant_exit': mutant, 'restored_exit': restored,
              'restored_source_sha256': hashlib.sha256(SOURCE.read_bytes()).hexdigest()}
    (args.output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    assert restored == 0
    print(json.dumps(report))


if __name__ == '__main__':
    main()
