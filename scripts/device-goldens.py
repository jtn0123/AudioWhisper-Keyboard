"""Golden files for the device comparisons (docs/phase6-kotlin-references.md, owed item E54): written
from the references on an emulator, then checked. python3 -I.

    python3 -I scripts/device-goldens.py OUTPUT write FAMILY        # at the family's generator commit
    python3 -I scripts/device-goldens.py OUTPUT check FAMILY [--runs N]

Needs ANDROID_SERIAL (never emulator-5554) and, for dictation and tts, the speech models already on
the device. Builds the family's fixture APKs, installs them with scripts/install-dev.sh and its
--probe option, and runs the family's instrumentation with `am instrument`, nothing else. `write`
asks the references' tapes to write (`-e referenceGolden write`), then pulls what they wrote from
the app's files/golden into app/src/androidTest/assets/golden. The app's stores, settings and
keyboard are checked unchanged after every run, and the keyboard set back.

Only golden names, answer counts and digests are printed; the instrumentation's own output goes to
OUTPUT. scripts/device-mutations.py runs faults through the same steps.
"""
import argparse
import hashlib
import os
from pathlib import Path
import re
import subprocess
import sys
from typing import NamedTuple, Tuple

ROOT = Path(__file__).resolve().parent.parent
PKG = 'com.jtn0123.audiowhisper.keyboard'
IME = PKG + '/dev.patrickgold.florisboard.FlorisImeService'
GOLDEN = ROOT / 'app/src/androidTest/assets/golden'
TEST_APK = 'app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk'
# What a run must leave as it found it: the user's stores, settings and keyboard.
KEPT = ('files/learned-words.txt', 'files/kept-as-typed.txt', 'files/tap-aim.txt', 'files/typing-counters.txt',
        'jetpref_datastore/florisboard-app-prefs.jetpref')


class Family(NamedTuple):
    """How one family's device comparison runs: the Gradle property that builds its runner, the
    runner, its arguments, and the line a passing run prints. (The debug build is shrunk; only a
    fixture build keeps what a test APK calls.)"""
    fixture: str
    runner: str
    arguments: Tuple[str, ...]
    passed: str


A1 = 'com.jtn0123.audiowhisper.suggest.A1Instrumentation'
FAMILIES = {
    'gif': Family('rendererFixture', 'com.jtn0123.audiowhisper.render.FixtureInstrumentation', ('-e', 'gifSearch', 'true'),
                  'gif_search=true'),
    'extension': Family('suggestFixture', A1, ('-e', 'only', 'extension'), 'INSTRUMENTATION_CODE: -1'),
    'boundary': Family('suggestFixture', A1, ('-e', 'boundaries', 'true'), 'INSTRUMENTATION_CODE: -1'),
    'a1': Family('suggestFixture', A1, ('-e', 'only', 'a1'), 'INSTRUMENTATION_CODE: -1'),
    'tts': Family('ttsFixture', 'com.jtn0123.audiowhisper.reader.TtsInstrumentation',
                  ('-e', 'stress', 'false', '-e', 'timings', 'false'), 'tts_complete=true'),
    'dictation': Family('dictationFixture', 'com.jtn0123.audiowhisper.dictation.DictationInstrumentation',
                        ('-e', 'reuse', 'true', '-e', 'timings', 'false'), 'dictation_complete=true'),
}


# Families whose reference cannot run on a device, so their goldens are written on the host and the
# device only checks them; how to write them.
HOST_WRITTEN = {
    'a1': "the reference's AOSP engine glue is host-only. Write them with REFERENCE_GOLDEN=write ./gradlew "
          ":app:testDebugUnitTest --rerun --tests '*A1SessionMatchTest' at the a1 generator commit, then move "
          "a1-*-main_en-US.txt from app/src/test/resources/golden to app/src/test/a1-goldens/golden",
}


def serial():
    value = os.environ.get('ANDROID_SERIAL', '')
    assert value and value != 'emulator-5554', 'set ANDROID_SERIAL to an emulator of your own, never emulator-5554'
    return value


def env():
    e = dict(os.environ)
    e.setdefault('CARGO_BUILD_JOBS', '6')
    if 'ANDROID_HOME' not in e:
        for line in (ROOT / 'local.properties').read_text().splitlines():
            if line.startswith('sdk.dir='):
                e['ANDROID_HOME'] = line.split('=', 1)[1]
    return e


def adb(*arguments, data=None):
    return subprocess.run(['adb', '-s', serial(), *arguments], check=True, capture_output=True,
                          input=data, timeout=120).stdout


def logged(output, name, command, timeout=3600):
    """Runs command with its output in OUTPUT/name.log; returns the exit and the log's text."""
    log = output / (name + '.log')
    with log.open('wb') as out:
        result = subprocess.run(command, cwd=ROOT, stdout=out, stderr=subprocess.STDOUT, env=env(), timeout=timeout)
    return result.returncode, log.read_text(errors='replace')


def state():
    """Digests of what a run must leave alone, and the keyboard in use."""
    digests = {}
    for path in KEPT:
        found = subprocess.run(['adb', '-s', serial(), 'exec-out', 'run-as', PKG, 'cat', path],
                               capture_output=True, timeout=120)
        digests[path] = hashlib.sha256(found.stdout).hexdigest() if found.returncode == 0 else None
    return digests


def build(output, family, name):
    """Builds the family's app and test APKs and installs them; asserts each step passed."""
    command = ['./gradlew', '--stacktrace', ':app:assembleDebug', ':app:assembleDebugAndroidTest', '-P' + family.fixture + '=true']
    status, _ = logged(output, name + '-build', command)
    assert status == 0, name + ': the build failed; see ' + name + '-build.log'
    status, _ = logged(output, name + '-install', ['scripts/install-dev.sh', serial()])
    assert status == 0, name + ': installing the app failed'
    status, _ = logged(output, name + '-test-install', ['scripts/install-dev.sh', '--probe', TEST_APK])
    assert status == 0, name + ': installing the test APK failed'


def instrument(output, family, name, *extra):
    """Runs the family's instrumentation; returns whether it passed and its log. Asserts the app's
    stores and settings are as they were, and sets the keyboard back."""
    before = state()
    command = ['adb', '-s', serial(), 'shell', 'am', 'instrument', '-w', *family.arguments, *extra,
               PKG + '.test/' + family.runner]
    try:
        _, log = logged(output, name, command)
    finally:
        adb('shell', 'ime', 'set', IME)
    assert state() == before, name + ': the run changed the app\'s stores or settings'
    passed = family.passed in log and 'INSTRUMENTATION_CODE: -1' in log
    return passed, log


def written():
    """The golden files the references' tapes wrote on the device, by name. (No `sh -c`: adb quotes
    each argument itself, so a quoted command would arrive as one word.)"""
    found = subprocess.run(['adb', '-s', serial(), 'exec-out', 'run-as', PKG, 'ls', 'files/golden'],
                           capture_output=True, timeout=120)
    return [name for name in found.stdout.decode().split() if re.fullmatch(r'[A-Za-z0-9._-]+\.txt', name)]


def summary(text):
    """A golden file's count and digest: what may be printed of it."""
    fields = dict(line.split(' ', 1) for line in text.splitlines() if line and not line.startswith('#')
                  and line.split(' ', 1)[0] in ('count', 'digest'))
    return f"count {fields['count']}, digest {fields['digest']}"


def write(output, family_name):
    assert family_name not in HOST_WRITTEN, f'{family_name} goldens are written on the host: {HOST_WRITTEN[family_name]}'
    family = FAMILIES[family_name]
    build(output, family, 'write')
    adb('exec-out', 'run-as', PKG, 'rm', '-rf', 'files/golden')
    passed, _ = instrument(output, family, 'write-run', '-e', 'referenceGolden', 'write')
    assert passed, 'the writing run failed; see write-run.log'
    names = written()
    assert names, 'the run wrote no golden files'
    GOLDEN.mkdir(parents=True, exist_ok=True)
    for name in names:
        text = adb('exec-out', 'run-as', PKG, 'cat', 'files/golden/' + name)
        (GOLDEN / name).write_bytes(text)
        print(f'{name}: {summary(text.decode())}')
    adb('exec-out', 'run-as', PKG, 'rm', '-rf', 'files/golden')
    print(f'{family_name}: {len(names)} golden files written to {GOLDEN.relative_to(ROOT)}')


def check(output, family_name, runs):
    family = FAMILIES[family_name]
    build(output, family, 'check')
    for run in range(1, runs + 1):
        passed, _ = instrument(output, family, f'check-run-{run}')
        assert passed, f'check run {run} failed; see check-run-{run}.log'
        print(f'{family_name}: check run {run} passed')


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('output', type=Path)
    parser.add_argument('mode', choices=('write', 'check'))
    parser.add_argument('family', choices=sorted(FAMILIES))
    parser.add_argument('--runs', type=int, default=2)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    if args.mode == 'write':
        write(args.output.resolve(), args.family)
    else:
        check(args.output.resolve(), args.family, args.runs)


if __name__ == '__main__':
    sys.dont_write_bytecode = True
    main()
