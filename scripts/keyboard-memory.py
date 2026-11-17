#!/usr/bin/env python3
"""Keyboard process memory, by dumpsys category, with the Rust keyboard renderer shown and typing.

ANDROID_SERIAL and AGENT_DEVICE must identify the owned aw_keyboard_ssd session.
python3 -I scripts/keyboard-memory.py OUTPUT PRIVATE_BASELINE
Run it right after installing, so every build starts from a fresh process.

Five points, in one process: settings open with the keyboard hidden, the keyboard shown in
the settings "Try out your setup" field, after 60 letter presses (no space, so nothing is
committed or learned), with colour-emoji candidates shown (the letters deleted, then an emoji
name typed with inline emoji suggestions on), and with the emoji panel open and scrolled. It saves dumpsys meminfo, the glyph-atlas-sized anonymous mappings from
smaps (a vello_cpu atlas page is 4096x4096 RGBA, 65,536 KiB) and every other anonymous mapping of 8 MiB or more, and the keyboard raster's per-frame
draw times. It never reads or saves the field's text.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import re
import statistics
import sys
sys.dont_write_bytecode = True
import time

SPEC = importlib.util.spec_from_file_location('settings_device', Path(__file__).parent / 'render-benchmark/settings_ui_device.py')
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
CATEGORIES = ('Native Heap', 'Dalvik Heap', 'Dalvik Other', 'Stack', 'Ashmem', 'Other dev', '.so mmap', '.jar mmap',
              '.apk mmap', '.ttf mmap', '.dex mmap', '.oat mmap', '.art mmap', 'Other mmap', 'GL mtrack', 'Graphics',
              'Unknown', 'TOTAL PSS')
ATLAS_KIB = 4096 * 4096 * 4 // 1024
LETTERS = 'qwertyuiopasdfghjklzxcvbnm'
# An emoji name with inline suggestions: the candidate row then draws colour glyphs.
EMOJI_QUERY = 'rocket'


def meminfo(d):
    text = d.adb('shell', 'dumpsys', 'meminfo', MODULE.PKG).decode()
    out = {}
    for line in text.splitlines():
        for name in CATEGORIES:
            match = re.match(r'\s*' + re.escape(name) + r':?\s+(\d+)', line)
            if match and name not in out:
                out[name] = int(match.group(1))
    return text, out


def large_mappings(d, pid):
    """Anonymous mappings of 8 MiB or more (an atlas page is 65,536 KiB plus allocator header)."""
    smaps = d.adb('exec-out', 'run-as', MODULE.PKG, 'cat', f'/proc/{pid}/smaps').decode(errors='replace')
    found, current = [], None
    for line in smaps.splitlines():
        if re.match(r'^[0-9a-f]+-[0-9a-f]+ ', line):
            name = line.split()[5] if len(line.split()) > 5 else ''
            current = {'name': name} if name == '' or name.startswith('[anon:') else None
            continue
        field = re.match(r'^(Size|Rss|Pss):\s+(\d+) kB', line)
        if current is not None and field:
            current[field.group(1).lower()] = int(field.group(2))
            if field.group(1) == 'Pss' and current['size'] >= 8192:
                current['atlas_sized'] = ATLAS_KIB <= current['size'] <= ATLAS_KIB + 64
                found.append(current)
    return found


def draw_times(d):
    log = d.adb('shell', 'logcat', '-d', '-s', 'aw-render:D', '*:S').decode(errors='replace')
    values = [int(m.group(1)) / 1e6 for m in re.finditer(r'frame draw_ns=(\d+)', log)]
    if not values:
        return {'frames': 0}
    values.sort()
    return {'frames': len(values), 'p50_ms': round(statistics.median(values), 3),
            'p95_ms': round(values[min(len(values) - 1, int(len(values) * .95))], 3), 'max_ms': round(values[-1], 3)}


def keys(d):
    """Letter keys' centres, from the keyboard's accessibility nodes."""
    out = {}
    for node in d.tree()['nodes']:
        label = (node.get('label') or '').strip()
        rect = node.get('rect')
        named = label in ('Delete', 'Emoji') or (len(label) == 1 and label.lower() in LETTERS)
        if named and rect and rect['y'] > 1200 and label.lower() not in out:
            out[label.lower()] = (rect['x'] + rect['width'] // 2, rect['y'] + rect['height'] // 2)
    return out


def colour_candidates(d):
    """Whether the candidate row shows an emoji. Only this boolean leaves the function."""
    return any(any(ord(c) >= 0x1F000 for c in (n.get('label') or '')) and (n.get('rect') or {}).get('y', 0) > 1200
               for n in d.tree()['nodes'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('baseline', type=Path)
    args = parser.parse_args()
    d = MODULE.SettingsDevice(args.output, args.baseline)
    assert d.adb('emu', 'avd', 'name').decode().splitlines()[0] == 'aw_keyboard_ssd'
    for stream, volume in [(1, 0), (2, 0), (3, 0), (5, 0), (4, 1), (10, 1)]:
        d.adb('shell', 'cmd', 'audio', 'set-volume', str(stream), str(volume))
        assert d.adb('shell', 'cmd', 'audio', 'get-stream-volume', str(stream)).decode().strip().endswith(f'-> {volume}')
    profile = d.adb('shell', 'getprop', 'log.tag.aw-profile').decode().strip()
    result = {}
    try:
        d.adb('shell', 'setprop', 'log.tag.aw-profile', 'DEBUG')
        d.fixture()
        # Inline emoji suggestions, through the app's own Emojis screen: a file written now would
        # race the IME process the system rebinds at once, which then saves its own copy.
        d.launch('settings/media')
        d.press('Trigger type', scroll=True)
        d.wait(lambda: d.find('Inline text'), 'Trigger type dialog absent')
        d.press('Inline text')
        if d.find('OK'):
            d.press('OK')
        d.wait(lambda: (d.prefs().get('emoji__suggestion_type') or '').strip('"') == 'INLINE_TEXT', 'Inline emoji suggestions did not persist')
        # Through the app's own switch, as d.prototype() does: the running process has already
        # read the preference file, so writing the file would not reach the keyboard.
        d.launch('settings/keyboard')
        d.wait(lambda: d.find('Number row'), 'Keyboard settings absent')
        if d.prefs().get('keyboard__rust_renderer_prototype', 'true') != 'true':
            d.press('Draw the keyboard in Rust')
            d.wait(lambda: d.prefs().get('keyboard__rust_renderer_prototype') == 'true', 'Prototype did not persist')
        pid = d.adb('shell', 'pidof', MODULE.PKG).decode().split()[0]
        time.sleep(3)
        for point in ('settings', 'shown', 'typed', 'emoji-candidates', 'emoji-panel'):
            d.adb('shell', 'logcat', '-c')
            if point == 'shown':
                d.ui('scroll', 'bottom')
                d.press('Try out your setup')
                d.wait(lambda: len(keys(d)) >= 20, 'Keyboard keys did not appear')
                time.sleep(3)
            if point == 'typed':
                centres = keys(d)
                for i in range(60):
                    x, y = centres[LETTERS[i % len(LETTERS)]]
                    d.ui('press', x, y)
                time.sleep(3)
            if point == 'emoji-candidates':
                centres = keys(d)
                for _ in range(60):
                    d.ui('press', *centres['delete'])
                for letter in EMOJI_QUERY:
                    d.ui('press', *centres[letter])
                d.wait(lambda: colour_candidates(d), 'No colour-emoji candidate appeared')
                time.sleep(3)
            if point == 'emoji-panel':
                # Clear the query so the actions row (with Emoji) replaces the candidates.
                for _ in EMOJI_QUERY:
                    d.ui('press', *keys(d)['delete'])
                d.wait(lambda: 'emoji' in keys(d), 'Emoji action absent')
                d.ui('press', *keys(d)['emoji'])
                time.sleep(2)
                # Swipes inside the panel (below the candidate row), never the settings page behind it.
                d.ui('swipe', 540, 2100, 540, 1750, '--count', 4, '--pause-ms', 600, '--pattern', 'ping-pong')
                time.sleep(3)
            assert d.adb('shell', 'pidof', MODULE.PKG).decode().split()[0] == pid, 'The process restarted'
            text, categories = meminfo(d)
            (d.output / f'{point}-meminfo.txt').write_text(text)
            result[point] = {'pss_kib': categories, 'large_mappings': large_mappings(d, pid), 'draw': draw_times(d)}
            print(point, json.dumps(result[point]), flush=True)
        d.save('keyboard-memory.json', result)
    finally:
        try:
            d.ui('back')
            d.adb('shell', 'setprop', 'log.tag.aw-profile', profile or "''")
            assert d.adb('shell', 'getprop', 'log.tag.aw-profile').decode().strip() == profile
        finally:
            d.cleanup()


if __name__ == '__main__':
    main()
