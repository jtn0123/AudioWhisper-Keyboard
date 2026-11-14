"""Canonical synthetic stores for the dedicated aw_keyboard test emulator.

python3 -I scripts/render-benchmark/store-session.py OUTPUT -- COMMAND [ARG ...]
No store contents are read, logged or saved. A pending record survives an interrupted host
process; rerunning installs the same fixture before any input. Device unavailability is a
hard failure, never a successful restoration. Keep agent-device idle during replay commands.
"""
import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess

PKG = 'com.jtn0123.audiowhisper.keyboard'
FIXTURE = Path(__file__).resolve().with_name('store-fixture')
NAMES = ('learned-words.txt', 'kept-as-typed.txt')


class Stores:
    def __init__(self, output):
        self.serial = os.environ['ANDROID_SERIAL']
        if self.serial == 'emulator-5554':
            raise ValueError('Reserved emulator')
        self.output = Path(output)
        self.output.mkdir(parents=True, exist_ok=True)
        self.expected = {name: hashlib.sha256((FIXTURE / name).read_bytes()).hexdigest() for name in NAMES}

    def adb(self, *args, **kwargs):
        return subprocess.run(['adb', '-s', self.serial, *args], check=True,
                              capture_output=True, timeout=30, **kwargs).stdout

    def install(self, phase):
        self.adb('shell', 'am', 'force-stop', PKG)
        for name in NAMES:
            self.adb('shell', 'run-as ' + PKG + ' sh -c ' + shlex.quote('cat > files/' + name),
                     input=(FIXTURE / name).read_bytes())
        actual = self.verify()
        (self.output / ('stores-' + phase + '.json')).write_text(json.dumps(actual, indent=2) + '\n')
        self.adb('shell', 'ime', 'set', PKG + '/dev.patrickgold.florisboard.FlorisImeService')

    def verify(self):
        actual = {name: self.adb('shell', 'run-as', PKG, 'sha256sum', 'files/' + name)
                  .decode().split()[0] for name in NAMES}
        if actual != self.expected:
            raise RuntimeError('Canonical store hash verification failed')
        return actual


@contextmanager
def fixture_session(output):
    stores = Stores(output)
    pending = stores.output / 'stores-pending.json'
    pending.write_text(json.dumps({'serial': stores.serial, 'expected': stores.expected}, indent=2) + '\n')
    try:
        stores.install('before')
        yield stores
    finally:
        stores.install('restored')
        stores.verify()
        pending.unlink()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output')
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    with fixture_session(args.output):
        if command:
            result = subprocess.run(command, check=False)
            if result.returncode:
                raise SystemExit(result.returncode)
    print('Canonical stores restored and hashes verified')


if __name__ == '__main__':
    main()
