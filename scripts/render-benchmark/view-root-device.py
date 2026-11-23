"""Device checks for the plain-View root (phase 3 item 4) on the build installed: each poses the
device as view-root-diff.py does, checks the input method window holds no composition, and
screenshots it into OUT. python3 -I; needs ANDROID_SERIAL and AGENT_DEVICE (every touch goes
through it). Records counts, classes and pixel tallies only: the probe's field stays empty, its
suggestions off, and the dictation preview is measured, never shown.

    python3 -I scripts/render-benchmark/view-root-device.py build/view-root/device [check ...]

The image themes come from image_theme(), a theme extension built here: the built-in day and
night stylesheets, each with a checkered background image on the window, put in the app's theme
directory as FLEX for those checks and taken out after.
Settings, the system's state and accessibility are put back as view-root-diff.py puts them.
"""
import argparse
import io
import json
import os
import runpy
import subprocess
import time
import zipfile
import zlib
from pathlib import Path

HERE = Path(__file__).resolve().parent
diff = runpy.run_path(str(HERE / 'view-root-diff.py'))
check = runpy.run_path(str(HERE / 'view-root-composition.py'))['check']
Device, PKG, DIMENSIONS = diff['Device'], diff['PKG'], diff['DIMENSIONS']
FLEX = 'files/ime/theme/test.viewroot.images.flex'
# The stripes' colours in each image, as bytes.
STRIPES = {'day': (b'\xff\xa0\x40', b'\xff\xf0\xd0'), 'night': (b'\x20\x40\xa0', b'\x05\x05\x20')}


def agent(*args, background=False):
    command = [os.environ['AGENT_DEVICE'], *map(str, args)]
    if background:
        return subprocess.Popen(command, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    subprocess.run(command, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=90)


def nodes(device):
    """The visible nodes of a dump that has them: one taken mid-change can come back without."""
    found = device.wait(lambda: device.tree().get('nodes'), 'Dump service gave no nodes', timeout=15)
    return [n for n in found if n.get('visible')]


def point(found, label):
    """The centre of the keyboard's key or action [label]: a key's label or an action's description."""
    for n in found:
        if label in (n.get('text') if n['class'].endswith('Key') else None, n.get('description')):
            x, y, right, bottom = n['bounds']
            return (x + right) // 2, (y + bottom) // 2
    raise RuntimeError('No keyboard node for a fixed label')


def screenshot(device, out, label):
    path = out / (label + '.png')
    with open(path, 'wb') as shot:
        subprocess.run(['adb', '-s', device.serial, 'exec-out', 'screencap', '-p'], stdout=shot, check=True)
    return path


def pixels(path):
    """The RGB rows of a screencap PNG (8-bit RGBA, any filter)."""
    data, at, idat = path.read_bytes(), 8, b''
    while at < len(data):
        n, kind = int.from_bytes(data[at:at + 4], 'big'), data[at + 4:at + 8]
        if kind == b'IHDR':
            w, h = int.from_bytes(data[at + 8:at + 12], 'big'), int.from_bytes(data[at + 12:at + 16], 'big')
        elif kind == b'IDAT':
            idat += data[at + 8:at + 8 + n]
        at += 12 + n
    raw, stride, rows, prev = zlib.decompress(idat), w * 4, [], bytearray(w * 4)
    for y in range(h):
        f, line = raw[y * (stride + 1)], bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for i in range(stride):
            a, b, c = (line[i - 4] if i >= 4 else 0), prev[i], (prev[i - 4] if i >= 4 else 0)
            p = a + b - c
            line[i] = (line[i] + (0, a, b, (a + b) // 2, a if abs(p - a) <= min(abs(p - b), abs(p - c))
                                  else b if abs(p - b) <= abs(p - c) else c)[f]) & 255
        rows.append(line)
        prev = line
    return rows


def tally(path, box, colours):
    """How many pixels in [box] are exactly one of [colours]; how many distinct colours it has."""
    rows, (x0, y0, x1, y1) = pixels(path), box
    found = [bytes(rows[y][x * 4:x * 4 + 3]) for y in range(y0, y1) for x in range(x0, x1)]
    return sum(px in colours for px in found), len(set(found))


def verify(device, out, label, results, **extra):
    ok, counts = check(out, 'views', device.serial, label)
    screenshot(device, out, label)
    results[label] = dict(counts, ok=ok, **extra)
    print(label, json.dumps(results[label], sort_keys=True), flush=True)
    return ok


def pose(device, raw, config):
    diff['pose'](device, dict({k: v[0] for k, v in DIMENSIONS.items()}, **config), raw)
    time.sleep(1.5)


def held(device, out, label, results, command, at):
    """Screenshots while agent-device holds a touch, [at] seconds after it begins."""
    hold = agent(*command, background=True)
    time.sleep(at)
    verify(device, out, label, results)
    hold.wait(timeout=60)


def fullscreen(device, raw, out, results):
    pose(device, raw, {'landscape': True, 'fullscreen': True})
    verify(device, out, 'fullscreen', results)
    found = nodes(device)
    held(device, out, 'fullscreen-popup', results, ('longpress', *point(found, 'e'), 6000), 2.5)
    held(device, out, 'fullscreen-bubble', results, ('press', *point(found, 'p'), '--hold-ms', 3500), 1.2)


def sheet(device, raw, out, results):
    pose(device, raw, {'prefs': {'s;gestures__space_bar_long_press': 'SHOW_SUBTYPE_PICKER'}})
    agent('longpress', *point(nodes(device), 'Space'), 1200)
    time.sleep(1.5)
    verify(device, out, 'subtype-sheet', results)
    agent('back')


def sheet_swapped(device, raw, out, results):
    """The sheet after the renderer was switched off and on in one process. A swapped-in sheet
    must sit as a fresh one does, so it is compared with `sheet`'s screenshot below the status
    bar, pixel for pixel; run `sheet` first."""
    pose(device, raw, {'prefs': {'s;gestures__space_bar_long_press': 'SHOW_SUBTYPE_PICKER'}})
    for renderer in ('compose', 'rust'):
        device.renderer(renderer)
        diff['show'](device)
    time.sleep(1.5)
    agent('longpress', *point(nodes(device), 'Space'), 1200)
    time.sleep(1.5)
    ok = verify(device, out, 'subtype-sheet-swapped', results)
    agent('back')
    fresh, swapped = pixels(out / 'subtype-sheet.png'), pixels(out / 'subtype-sheet-swapped.png')
    # Below the input method window's top: the status bar's clock and icons differ by the minute.
    top = diff['window'](device)['frame'][1]
    differing = sum(sum(1 for x in range(0, len(fresh[y]), 4) if fresh[y][x:x + 3] != swapped[y][x:x + 3])
                    for y in range(top, len(fresh)) if fresh[y] != swapped[y])
    results['subtype-sheet-swapped'].update(differing=differing, ok=ok and differing == 0)
    print('subtype-sheet-swapped differing pixels:', differing, flush=True)


def images(device, raw, out, results):
    flex = image_theme()
    subprocess.run(['adb', '-s', device.serial, 'shell', f'run-as {PKG} sh -c "mkdir -p files/ime/theme && cat > {FLEX}"'],
                   input=flex, check=True)
    try:
        for mode in ('day', 'night'):
            pose(device, raw, {'prefs': {'s;theme__mode': 'ALWAYS_' + mode.upper(),
                                         's;theme__day_theme_id': 'test.viewroot.images:img_day',
                                         's;theme__night_theme_id': 'test.viewroot.images:img_night'}})
            time.sleep(1.5)
            frame = diff['window'](device)['frame']
            verify(device, out, 'image-' + mode, results)
            striped, distinct = tally(out / f'image-{mode}.png', frame, STRIPES[mode])
            other, _ = tally(out / f'image-{mode}.png', frame, STRIPES['night' if mode == 'day' else 'day'])
            results['image-' + mode].update(striped=striped, other=other, distinct=distinct,
                                            ok=results['image-' + mode]['ok'] and striped > 1000 and other == 0)
            print('image-' + mode, 'striped', striped, 'other', other, flush=True)
    finally:
        device.adb('shell', 'run-as', PKG, 'rm', '-f', FLEX)


def rotation(device, raw, out, results):
    pose(device, raw, {})
    pid = device.adb('shell', 'pidof', PKG).strip()
    for step, lock in (('rotated', '1'), ('rotated-back', '0')):
        device.adb('shell', 'wm', 'user-rotation', 'lock', lock)
        diff['settle'](device)
        time.sleep(1.5)
        shown = 'mInputShown=true' in device.adb('shell', 'dumpsys', 'input_method')
        verify(device, out, step, results, shown=shown, same_process=device.adb('shell', 'pidof', PKG).strip() == pid)


def pid_of(device):
    """The keyboard's process id, or '' while it has none (pidof then exits 1)."""
    return subprocess.run(['adb', '-s', device.serial, 'shell', 'pidof', PKG], capture_output=True, text=True).stdout.strip()


def death(device, raw, out, results):
    pose(device, raw, {})
    pid = pid_of(device)
    device.adb('shell', 'run-as', PKG, 'kill', '-9', pid)
    device.wait(lambda: pid_of(device) != pid, 'Keyboard was not killed', timeout=30)
    diff['show'](device)
    device.wait(lambda: pid_of(device) not in ('', pid), 'Keyboard did not come back', timeout=60)
    time.sleep(1.5)
    verify(device, out, 'process-death', results, new_process=pid_of(device) not in ('', pid))


def gif(device, raw, out, results):
    """The media panel's GIF tab on the fixture build (-PgifFixture=true). The fixture's source
    lives in the keyboard process only, and posing restarts that process, so it is switched in
    after the pose."""
    pose(device, raw, {})
    subprocess.run(['python3', '-I', str(Path(__file__).parents[1] / 'gif-fixture.py'), 'push', device.serial],
                   check=True, stdout=subprocess.DEVNULL)
    agent('press', *point(nodes(device), 'Emoji'))
    time.sleep(1.5)
    found = nodes(device)
    tab = next(n for n in found if 'GIF' in ((n.get('description') or '') + (n.get('text') or '')).upper())
    x, y, right, bottom = tab['bounds']
    agent('press', (x + right) // 2, (y + bottom) // 2)
    time.sleep(2)
    verify(device, out, 'gif-panel', results)


def blank_above(path, bottom):
    """Rewrites the screencap at [path] grey above row [bottom], so nothing typed or dictated stays."""
    rows = pixels(path)
    for y in range(0, bottom):
        rows[y][:] = b'\x80\x80\x80\xff' * (len(rows[y]) // 4)
    path.write_bytes(png(rows, 4))


def png(rows, channels):
    """A PNG of [rows] of 8-bit RGB (3 [channels]) or RGBA (4) pixels, unfiltered."""
    raw_rows = b''.join(b'\x00' + bytes(r) for r in rows)
    chunk = lambda k, b: len(b).to_bytes(4, 'big') + k + b + zlib.crc32(k + b).to_bytes(4, 'big')
    head = (len(rows[0]) // channels).to_bytes(4, 'big') + len(rows).to_bytes(4, 'big') + bytes([8, 6 if channels == 4 else 2, 0, 0, 0])
    return b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', head) + chunk(b'IDAT', zlib.compress(raw_rows, 6)) + chunk(b'IEND', b'')


def image_theme():
    """The image-theme fixture as .flex bytes: the built-in day and night stylesheets, each with
    a `background-image` on `window` (360 by 200, 30 px squares in that mode's two STRIPES
    colours), under a manifest made from the built-in extension's. The zip carries a
    `stylesheets/` directory entry: the app's unzip makes no parent directories, and without it
    the theme silently falls back to the base style."""
    base = HERE.parents[1] / 'app/src/main/assets/ime/theme/org.florisboard.themes'
    manifest = json.loads((base / 'extension.json').read_text())
    authors = manifest['themes'][0]['authors']
    manifest['meta'].update(id='test.viewroot.images', title='View root image test', description='Device check fixture')
    manifest['themes'] = [{'id': 'img_' + mode, 'label': 'Image ' + mode.title(), 'authors': authors,
                           'isNight': mode == 'night'} for mode in ('day', 'night')]
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, 'w', zipfile.ZIP_DEFLATED) as flex:
        flex.writestr('extension.json', json.dumps(manifest, indent=2))
        flex.mkdir('stylesheets')
        for mode in ('day', 'night'):
            first, second = STRIPES[mode]
            flex.writestr(f'bg-{mode}.png', png([b''.join(second if (x // 30 + y // 30) % 2 == 0 else first
                                                          for x in range(360)) for y in range(200)], 3))
        for mode in ('day', 'night'):
            sheet = json.loads((base / 'stylesheets' / f'floris_{mode}.json').read_text())
            sheet['window'] = dict(sheet['window'], **{'background-image': f'uri(`flex:/bg-{mode}.png`)', 'content-scale': 'crop'})
            flex.writestr(f'stylesheets/img_{mode}.json', json.dumps(sheet, indent=2))
    return buffer.getvalue()


def dictation(device, raw, out, results):
    """The mic action, replaying the debug build's fixture WAV, never the microphone: without the
    WAV the build would open the mic, so the check refuses to start. Only the strip's pixel tally
    and the field's length are kept; the screenshots blank everything above the keys."""
    if subprocess.run(['adb', '-s', device.serial, 'shell', 'run-as', PKG, 'test', '-f', 'files/debug-mic.wav']).returncode:
        raise RuntimeError('No fixture recording (files/debug-mic.wav): dictation would open the microphone')
    pose(device, raw, {})
    found = nodes(device)
    window = diff['window'](device)
    keys_top = min(n['bounds'][1] for n in found if n['class'].endswith('Key'))
    strip = (0, window['content_top'], window['frame'][2], keys_top)
    try:
        before = screenshot(device, out, 'dictation-before')
        agent('press', *point(found, 'Voice input'))
        time.sleep(6)
        ok, counts = check(out, 'views', device.serial, 'dictation-preview')
        during = screenshot(device, out, 'dictation-preview')
        changed = sum(a != b for a, b in zip(*(pixels(p)[strip[1]:strip[3]] for p in (before, during))))
        # The dump service can give no nodes while the level meter redraws every frame: then
        # dictation is left to end by itself after the recording's silence.
        tree = device.tree()
        stop = next((n for n in tree.get('nodes') or [] if n.get('visible') and 'stop' in (n.get('description') or '').lower()), None)
        if stop:
            x, y, right, bottom = stop['bounds']
            agent('press', (x + right) // 2, (y + bottom) // 2)
        device.wait(lambda: device.digest()['utf16_units'] > 0, 'Dictation typed nothing', timeout=30)
        units = device.digest()['utf16_units']
        results['dictation'] = dict(counts, ok=ok and changed > 0 and units > 0, strip_rows_changed=changed,
                                    stop_found=stop is not None, dump_keys=sorted(tree), field_units=units)
        print('dictation', json.dumps(results['dictation'], sort_keys=True), flush=True)
    finally:
        for name in ('dictation-before', 'dictation-preview'):
            if (out / (name + '.png')).exists():
                blank_above(out / (name + '.png'), strip[3])
    device.reset(True)


SIMPLE = {
    'portrait': {}, 'landscape': {'landscape': True},
    'buttons-portrait': {'nav': 'buttons'}, 'buttons-landscape': {'nav': 'buttons', 'landscape': True},
    'one-handed-start': {'one_handed': 'START'}, 'one-handed-end': {'one_handed': 'END'},
    'font-1.3': {'font_scale': '1.3'}, 'font-2.0': {'font_scale': '2.0'},
}
STEPS = {'fullscreen': fullscreen, 'sheet': sheet, 'sheet-swapped': sheet_swapped, 'images': images,
         'rotation': rotation, 'death': death, 'gif': gif, 'dictation': dictation}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('out')
    parser.add_argument('checks', nargs='*')
    args = parser.parse_args()
    device = Device(args.out)
    out = Path(args.out)
    raw, state = diff['saved'](out, device)
    results = {}
    try:
        device.mute()
        for label in args.checks or [*SIMPLE, *STEPS]:
            if label in SIMPLE:
                pose(device, raw, SIMPLE[label])
                verify(device, out, label, results)
            else:
                STEPS[label](device, raw, out, results)
    finally:
        diff['restore'](device, raw, state['system'], state['access'])
        gone = subprocess.run(['adb', '-s', device.serial, 'shell', 'run-as', PKG, 'test', '-e', FLEX]).returncode != 0
        print('image theme removed:', gone, flush=True)
        (out / 'results.json').write_text(json.dumps(results, indent=1, sort_keys=True) + '\n')
        for private in ('private-state.json', 'private-prefs.bin'):
            (out / private).unlink()
    failed = sorted(k for k, v in results.items() if not v['ok'])
    print('failed:', failed)
    raise SystemExit(1 if failed else 0)


if __name__ == '__main__':
    main()
