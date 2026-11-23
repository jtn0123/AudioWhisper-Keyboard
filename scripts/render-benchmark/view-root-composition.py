"""Runtime proof that the input method window holds no composition while the Rust renderer is on.

With the keyboard showing, dumps the visible windows' view hierarchies
(`cmd window dump-visible-window-views`) into OUT and reads only class names from the
InputMethod window's: never text or any other property. `--expect views` passes when no
`androidx.compose` class is there and ImeRootView is; `--expect compose` passes when one is,
which shows the check sees a composition when one is loaded (the fallback).

    ANDROID_SERIAL=emulator-5558 python3 -I view-root-composition.py build/view-root/views --expect views
"""
import argparse
import json
import os
import re
import subprocess
import sys
import zipfile
from pathlib import Path

CLASS = re.compile(rb'(?:android|androidx|com|dev|org)(?:\.[A-Za-z_][\w$]*)+')
ROOT = 'com.jtn0123.audiowhisper.render.ImeRootView'


def classes(dump):
    """The class names in the InputMethod window's hierarchy, and how many such windows there were."""
    with zipfile.ZipFile(dump) as archive:
        windows = [n for n in archive.namelist() if n.endswith(' InputMethod')]
        found = set()
        for window in windows:
            data = archive.read(window)
            found.update(name for m in CLASS.finditer(data) if (name := prefixed(data, m)))
    return found, len(windows)


def prefixed(data, match):
    """The string [match] starts, by the 2-byte length ViewDebug's encoder writes before it."""
    start = match.start()
    length = int.from_bytes(data[start - 2:start], 'big') if start >= 2 else 0
    return data[start:start + length].decode() if 0 < length <= len(match.group()) else None


def check(out, expect, serial, label=None):
    """Dumps the windows into [out] and checks [expect]; returns whether it held and the counts."""
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    dump = out / ((label or expect) + '.zip')
    with open(dump, 'wb') as file:
        subprocess.run(['adb', '-s', serial, 'exec-out', 'cmd', 'window', 'dump-visible-window-views'],
                       stdout=file, check=True)
    found, windows = classes(dump)
    compose = [c for c in found if c.startswith('androidx.compose')]
    counts = {'windows': windows, 'classes': len(found), 'compose': len(compose),
              'root': ROOT in found}
    ok = windows == 1 and (ROOT in found and not compose if expect == 'views' else bool(compose))
    return ok, counts


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('out')
    parser.add_argument('--expect', choices=('views', 'compose'), required=True)
    args = parser.parse_args()
    ok, counts = check(args.out, args.expect, os.environ['ANDROID_SERIAL'])
    print(json.dumps(counts, sort_keys=True), 'expect', args.expect + ':', 'pass' if ok else 'FAIL')
    sys.exit(0 if ok else 1)


if __name__ == '__main__':
    main()
