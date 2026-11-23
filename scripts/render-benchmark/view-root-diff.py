"""Seeded differential for the plain-View root (phase 3 item 4): the keyboard's bounds and touch
region under seeded window configurations, captured on one build and compared with another's.
python3 -I; needs ANDROID_SERIAL and AGENT_DEVICE (the device_open command: no adb taps).

    python3 -I scripts/render-benchmark/view-root-diff.py capture build/view-root/diff/branch --seed 7 --count 12
    python3 -I scripts/render-benchmark/view-root-diff.py compare build/view-root/diff/main build/view-root/diff/branch
    python3 -I scripts/render-benchmark/view-root-diff.py pose build/view-root/hold '{"landscape": true}'
    python3 -I scripts/render-benchmark/view-root-diff.py restore build/view-root/hold
    python3 -I scripts/render-benchmark/view-root-diff.py swap build/view-root/swap

Only geometry is recorded: bounds, classes, key labels and content descriptions, never a
field's text, a suggestion or a settings value beyond the fixture's own. Settings, rotation,
navigation mode, display cutout and font scale are put back, the settings byte for byte.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import re
import runpy
import shlex
import subprocess
import time

HERE = Path(__file__).resolve().parent
api = runpy.run_path(str(HERE / 'a11y_device.py'))
Device, PKG, PREFS, SERVICE = (api[k] for k in ('Device', 'PKG', 'PREFS', 'SERVICE'))
IME = PKG + '/dev.patrickgold.florisboard.FlorisImeService'
NAV = {'gesture': 'com.android.internal.systemui.navbar.gestural',
       'buttons': 'com.android.internal.systemui.navbar.threebutton'}
CUTOUT = 'com.android.internal.display.cutout.emulation.tall'
# Each dimension's values; a configuration takes one of each.
DIMENSIONS = {
    'landscape': [False, True],
    'nav': ['gesture', 'buttons'],
    'cutout': [False, True],
    'height': [100, 80, 125],
    'offset': [0, 12],
    'one_handed': [None, 'START', 'END'],
    'font_scale': ['1.0', '1.3', '2.0'],
    'fullscreen': [False, True],
}
# The configurations every run checks, whatever the seed.
FIXED = [
    {},
    {'landscape': True},
    {'landscape': True, 'fullscreen': True},
    {'one_handed': 'START'},
    {'one_handed': 'END', 'font_scale': '2.0'},
    {'nav': 'buttons', 'height': 125, 'offset': 12},
]


def configs(seed, count):
    rng = random.Random(seed)
    made = [dict({k: v[0] for k, v in DIMENSIONS.items()}, **fixed) for fixed in FIXED]
    while len(made) < len(FIXED) + count:
        config = {k: rng.choice(v) for k, v in DIMENSIONS.items()}
        if config not in made:
            made.append(config)
    return made


def name(config):
    return hashlib.sha256(json.dumps(config, sort_keys=True).encode()).hexdigest()[:10]


def prefs_for(config):
    side = config['one_handed']
    return {'b;keyboard__rust_renderer_prototype': True, 'b;keyboard__one_handed_mode_enabled': side is not None,
            's;keyboard__one_handed_mode': side or 'END', 'i;keyboard__one_handed_mode_scale_factor': 87,
            'i;keyboard__height_factor_portrait': config['height'], 'i;keyboard__height_factor_landscape': config['height'],
            'i;keyboard__bottom_offset_portrait': config['offset'], 'i;keyboard__bottom_offset_landscape': config['offset'],
            's;keyboard__landscape_input_ui_mode': 'ALWAYS_SHOW' if config['fullscreen'] else 'NEVER_SHOW',
            'b;suggestion__enabled': False, 'b;emoji__suggestion_enabled': False}


def read_prefs(device):
    return subprocess.check_output(['adb', '-s', device.serial, 'exec-out', 'run-as', PKG, 'cat', PREFS])


def write_prefs(device, values, raw):
    keys = {key.split(';', 1)[1] for key in values}
    rows = [row for row in raw.decode().splitlines() if len(p := row.split(';', 2)) != 3 or p[1] not in keys]
    rows += [key + ';' + json.dumps(value, separators=(',', ':')) for key, value in values.items()]
    device.adb('shell', 'am', 'force-stop', PKG)
    subprocess.run(['adb', '-s', device.serial, 'shell', 'run-as ' + PKG + ' sh -c ' + shlex.quote('cat > ' + PREFS)],
                   input=('\n'.join(rows) + '\n').encode(), check=True)
    device.adb('shell', 'ime', 'set', IME)


def system(device):
    get = lambda *a: device.adb('shell', 'settings', 'get', *a).strip()
    overlays = device.adb('shell', 'cmd', 'overlay', 'list')
    return {'font_scale': get('system', 'font_scale'), 'user_rotation': get('system', 'user_rotation'),
            'accelerometer_rotation': get('system', 'accelerometer_rotation'),
            'nav': next((k for k, v in NAV.items() if '[x] ' + v in overlays), 'gesture'),
            'cutout': '[x] ' + CUTOUT in overlays}


def put_system(device, state):
    put = lambda ns, key, value: device.adb('shell', 'settings', *(['delete', ns, key] if value == 'null' else ['put', ns, key, value]))
    # Writing user_rotation's own value again rotates nothing; the window manager's lock does.
    device.adb('shell', 'wm', 'user-rotation', 'lock', state['user_rotation'])
    put('system', 'accelerometer_rotation', state['accelerometer_rotation'])
    put('system', 'user_rotation', state['user_rotation'])
    if state['accelerometer_rotation'] == '1':
        device.adb('shell', 'wm', 'user-rotation', 'free')
    else:
        device.wait(lambda: 'mCurrentRotation=ROTATION_' + {'0': '0', '1': '90', '2': '180', '3': '270'}[state['user_rotation']]
                    in device.adb('shell', 'dumpsys', 'window', 'displays'), 'Display did not rotate')
    put('system', 'font_scale', state['font_scale'])
    device.adb('shell', 'cmd', 'overlay', 'enable-exclusive', '--category', NAV[state['nav']])
    device.adb('shell', 'cmd', 'overlay', 'enable' if state['cutout'] else 'disable', CUTOUT)
    time.sleep(2)


def window(device):
    dump = device.adb('shell', 'dumpsys', 'window', 'InputMethod')
    region = re.search(r'touchable region=SkRegion\(((?:\(\d+,\d+,\d+,\d+\))*)\)', dump)
    insets = re.search(r'mGivenContentInsets=\[(\d+),(\d+)\]\[(\d+),(\d+)\] mGivenVisibleInsets=\[(\d+),(\d+)\]', dump)
    frame = re.search(r'mFrame=\[(-?\d+),(-?\d+)\]\[(-?\d+),(-?\d+)\]', dump)
    return {'touchable': [list(map(int, r)) for r in re.findall(r'\((\d+),(\d+),(\d+),(\d+)\)', region.group(1))] if region else None,
            'content_top': int(insets.group(2)) if insets else None, 'visible_top': int(insets.group(6)) if insets else None,
            'frame': list(map(int, frame.groups())) if frame else None}


def geometry(nodes):
    """The keyboard's own nodes: the surface, its keys and the chrome beside it, by bounds."""
    ime = [n for n in nodes if n.get('visible') and n['class'] in
           ('android.view.SurfaceView', 'android.inputmethodservice.Keyboard$Key', 'android.widget.Button', 'android.view.View')]
    rows = sorted([n['class'], n.get('description') or '', (n.get('text') or '') if n['class'].endswith('Key') else '',
                   n['bounds']] for n in ime)
    surface = [n['bounds'] for n in ime if n['class'] == 'android.view.SurfaceView']
    keys = [n['bounds'] for n in ime if n['class'].endswith('Key')]
    union = [min(b[0] for b in keys), min(b[1] for b in keys), max(b[2] for b in keys), max(b[3] for b in keys)] if keys else None
    return {'surface': surface, 'keys': len(keys), 'key_union': union, 'nodes': rows}


def pose(device, config, raw):
    """Puts the device in [config] with the probe's field focused and the keyboard up; a device
    check may add settings of its own as `prefs`, rows' `type;key` to value."""
    write_prefs(device, dict(prefs_for(config), **config.get('prefs', {})), raw)
    # The launcher holds portrait; the window manager rotates only once the probe is on top.
    device.shell('am', 'start', '-W', '-n', api['PROBE'] + '/.ProbeActivity')
    put_system(device, {'font_scale': config['font_scale'], 'accelerometer_rotation': '0',
                        'user_rotation': '1' if config['landscape'] else '0', 'nav': config['nav'], 'cutout': config['cutout']})
    print(name(config), 'starting', flush=True)
    show(device)


def show(device):
    """The probe's field focused, so the keyboard is up."""
    try:
        device.probe(True)
    except RuntimeError as failure:
        # The probe taps its field to focus it; a keyboard already up takes that tap.
        if str(failure) != 'Probe not empty':
            raise
        device.reset(True)
    device.services(SERVICE)


def restore(device, raw, original, access):
    """Puts back the settings byte for byte, the system's state and accessibility."""
    # The probe's field goes first: a field without suggestions expands the Smartbar's actions.
    device.adb('shell', 'am', 'force-stop', api['PROBE'])
    put_system(device, original)
    device.restore_accessibility(access)
    device.adb('shell', 'am', 'force-stop', PKG)
    subprocess.run(['adb', '-s', device.serial, 'shell', 'run-as ' + PKG + ' sh -c ' + shlex.quote('cat > ' + PREFS)],
                   input=raw, check=True)
    device.adb('shell', 'ime', 'set', IME)
    time.sleep(3)
    print('settings restored byte for byte:', read_prefs(device) == raw, '; system restored:', system(device) == original)


def saved(out, device):
    """The state before the first pose in [out], saved there once, private."""
    state = out / 'private-state.json'
    if not state.exists():
        state.write_text(json.dumps({'system': system(device), 'access': device.original_accessibility()}))
        (out / 'private-prefs.bin').write_bytes(read_prefs(device))
    return (out / 'private-prefs.bin').read_bytes(), json.loads(state.read_text())


def capture(args):
    device = Device(args.out)
    out = Path(args.out)
    raw, state = saved(out, device)
    results = {}
    try:
        device.mute()
        for config in configs(args.seed, args.count):
            if args.only and name(config) not in args.only.split(','):
                continue
            pose(device, config, raw)
            data = device.capture(name(config), lambda ns: any(n['class'].endswith('Key') for n in ns))
            time.sleep(.5)
            results[name(config)] = {'config': config, 'window': window(device), 'ime': geometry(data['nodes'])}
            print(name(config), results[name(config)]['ime']['keys'], 'keys', flush=True)
    finally:
        restore(device, raw, state['system'], state['access'])
        (out / 'geometry.json').write_text(json.dumps(results, indent=1, sort_keys=True) + '\n')
        for private in ('private-state.json', 'private-prefs.bin'):
            (out / private).unlink()


def swap(args):
    """The renderer setting switched while the service lives: the Rust root, the Compose fallback,
    then the Rust root again, each checked for a composition and screenshot into [out]."""
    check = runpy.run_path(str(HERE / 'view-root-composition.py'))['check']
    device = Device(args.out)
    out = Path(args.out)
    raw, state = saved(out, device)
    failed = False
    try:
        device.mute()
        pose(device, {k: v[0] for k, v in DIMENSIONS.items()}, raw)
        pid = device.adb('shell', 'pidof', PKG).strip()
        for step, (renderer, expect) in enumerate((('rust', 'views'), ('compose', 'compose'), ('rust', 'views'))):
            device.renderer(renderer)
            show(device)
            time.sleep(2)
            ok, counts = check(out, expect, device.serial, f'{step}-{renderer}')
            with open(out / f'{step}-{renderer}.png', 'wb') as shot:
                subprocess.run(['adb', '-s', device.serial, 'exec-out', 'screencap', '-p'], stdout=shot, check=True)
            alive = device.adb('shell', 'pidof', PKG).strip() == pid
            failed |= not (ok and alive)
            print(step, renderer, json.dumps(counts, sort_keys=True), 'expect', expect, 'pass' if ok else 'FAIL',
                  '; same process:', alive, flush=True)
    finally:
        restore(device, raw, state['system'], state['access'])
        for private in ('private-state.json', 'private-prefs.bin'):
            (out / private).unlink()
    raise SystemExit(1 if failed else 0)


def hold(args):
    """`pose`: one configuration left on screen for a device check; `restore` ends it."""
    device = Device(args.out)
    out = Path(args.out)
    raw, state = saved(out, device)
    if args.command == 'pose':
        device.mute()
        pose(device, dict({k: v[0] for k, v in DIMENSIONS.items()}, **json.loads(args.config)), raw)
    else:
        restore(device, raw, state['system'], state['access'])
        for private in ('private-state.json', 'private-prefs.bin'):
            (out / private).unlink()


def containers(capture):
    """Takes out the input view's own container nodes: Compose's views, which the plain root has
    not (an intended difference), counted."""
    count = 0
    for entry in capture.values():
        rows = entry['ime']['nodes']
        kept = [row for row in rows if not (row[0] == 'android.view.View' and row[1:3] == ['', ''])]
        count += len(rows) - len(kept)
        entry['ime']['nodes'] = kept
    return count


def compare(args):
    a, b = (json.loads((Path(p) / 'geometry.json').read_text()) for p in (args.a, args.b))
    held = {args.a: containers(a), args.b: containers(b)}
    same, differ = 0, []
    for key in sorted(a.keys() & b.keys()):
        fields = [f for f in ('window', 'ime') if a[key][f] != b[key][f]]
        if fields:
            detail = {f: sorted(k for k in a[key][f] if a[key][f][k] != b[key][f].get(k)) for f in fields}
            differ.append({'config': a[key]['config'], 'differs': detail})
        else:
            same += 1
    report = {'configs': len(a.keys() & b.keys()), 'identical': same, 'different': differ, 'containers': held,
              'missing': sorted(a.keys() ^ b.keys())}
    print(json.dumps(report, indent=1, sort_keys=True))
    raise SystemExit(0 if not differ and not report['missing'] else 1)


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest='command', required=True)
    c = sub.add_parser('capture')
    c.add_argument('out')
    c.add_argument('--seed', type=int, default=7)
    c.add_argument('--count', type=int, default=12)
    c.add_argument('--only', help='comma-separated configuration names to capture')
    p = sub.add_parser('pose')
    p.add_argument('out')
    p.add_argument('config', help='JSON overrides of the baseline, e.g. {"landscape": true}')
    r = sub.add_parser('restore')
    r.add_argument('out')
    w = sub.add_parser('swap')
    w.add_argument('out')
    k = sub.add_parser('compare')
    k.add_argument('a')
    k.add_argument('b')
    args = parser.parse_args()
    if args.command != 'compare' and not os.environ.get('AGENT_DEVICE'):
        raise SystemExit('Set AGENT_DEVICE to the device_open command: taps go through it, never adb')
    {'capture': capture, 'pose': hold, 'restore': hold, 'swap': swap, 'compare': compare}[args.command](args)


if __name__ == '__main__':
    main()
