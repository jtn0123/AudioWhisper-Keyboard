"""Enable legacy callbacks in a normal APK deliberately; require the packaged-export check to fail."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / 'lib/rust/crates/awnative/Cargo.toml'
APK = ROOT / 'app/build/outputs/apk/debug/app-debug.apk'


def build(output, phase):
    # Verify fresh package bytes, rather than accepting stale incremental ZIP entries.
    shutil.rmtree(ROOT / 'app/build/intermediates/incremental/packageDebug', ignore_errors=True)
    APK.unlink(missing_ok=True)
    with (output / (phase + '-build.log')).open('wb') as log:
        result = subprocess.run(['./gradlew', '--stacktrace', 'assembleDebug'], cwd=ROOT,
                                stdout=log, stderr=subprocess.STDOUT)
    if result.returncode:
        raise RuntimeError('Export mutation must compile; see build log')


def check(output, phase):
    with (output / (phase + '-check.log')).open('wb') as log:
        return subprocess.run(['python3', '-I', 'scripts/consolidation-apk-check.py', str(APK)],
                              cwd=ROOT, stdout=log, stderr=subprocess.STDOUT).returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    original = MANIFEST.read_bytes()
    (args.output / 'original.toml').write_bytes(original)
    try:
        text = original.decode()
        assert text.count('[features]\n') == 1
        MANIFEST.write_text(text.replace('[features]\n', '[features]\ndefault = ["legacy-fixture"]\n'))
        build(args.output, 'mutant')
        mutant = check(args.output, 'mutant')
        assert mutant != 0
        assert 'legacy callback exports' in (args.output / 'mutant-check.log').read_text()
    finally:
        MANIFEST.write_bytes(original)
        assert MANIFEST.read_bytes() == original
    build(args.output, 'restored')
    restored = check(args.output, 'restored')
    report = {'mutant_check_exit': mutant, 'restored_check_exit': restored,
              'restored_sha256': hashlib.sha256(MANIFEST.read_bytes()).hexdigest()}
    (args.output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    assert restored == 0
    print(json.dumps(report))


if __name__ == '__main__':
    main()
