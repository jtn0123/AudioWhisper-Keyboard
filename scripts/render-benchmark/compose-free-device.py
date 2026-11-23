"""Device checks for the Compose-free render path: every panel that draws an icon, light and dark,
Material You on several seeds and flag sets, a theme font, a broken and a missing theme font, and
the Compose fallback with the renderer switched off. Each poses the device as view-root-diff.py
does, checks the input method window's composition and screenshots it into OUT; `compare` then
counts the pixels that differ inside the input method window between two such runs (main's build
and the branch's). python3 -I; needs ANDROID_SERIAL and AGENT_DEVICE (every touch goes through
it), and a -PgifFixture=true build for the GIF checks.

    python3 -I scripts/render-benchmark/compose-free-device.py run build/compose-free/device/branch [check ...]
    python3 -I scripts/render-benchmark/compose-free-device.py compare build/compose-free/device/main build/compose-free/device/branch

The themes come from fixture_theme(), a theme extension built here and put in the app's theme
directory as FLEX for the run, taken out after. The clipboard panel's screenshot is greyed below
its header before anything reads it, so no clipboard item is kept; the probe's field stays empty
and suggestions off. Settings, the system's state and accessibility are put back as
view-root-diff.py puts them, the settings byte for byte.
"""
import argparse
import io
import json
import runpy
import subprocess
import time
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
dev = runpy.run_path(str(HERE / 'view-root-device.py'))
diff, check, PKG = dev['diff'], dev['check'], dev['PKG']
agent, nodes, pixels, png = dev['agent'], dev['nodes'], dev['pixels'], dev['png']
ROOT = HERE.parents[1]
THEMES = ROOT / 'app/src/main/assets/ime/theme'
FONTS = ROOT / 'lib/rust/crates/render/tests/fixtures'
EXT = 'test.composefree'
FLEX = f'files/ime/theme/{EXT}.flex'
BUILT_IN = {'day': 'org.florisboard.themes:floris_day', 'night': 'org.florisboard.themes:floris_night'}
# Material You cases: a seed (sRGB ARGB) and the flags of the fixture's component for it.
SEEDS = {
    'purple': (0xff6750a4, ('TonalSpot', 'Default', 'SPEC_2021')),
    'teal': (0xff00897b, ('Expressive', 'High', 'SPEC_2025')),
    'pink': (0xffe91e63, ('Rainbow', 'Reduced', 'SPEC_2021')),
    'amber': (0xffffb300, ('Fidelity', 'Medium', 'SPEC_2025')),
}
# Font cases: the face file each `@font` rule's src names, and whether the flex holds it.
FONT_CASES = {'font': 'regular.ttf', 'font-broken': 'broken.ttf', 'font-missing': 'missing.ttf'}


def accent(argb):
    """`theme__accent_color` as the app writes it: a Compose Color's packed value in hex."""
    return f'{argb:08x}00000000'


def theme(mode, ident=None, **prefs):
    """Settings for [mode] ('day' or 'night') with that mode's theme set to [ident]."""
    ids = dict(BUILT_IN, **({mode: ident} if ident else {}))
    return dict({'s;theme__mode': 'ALWAYS_' + mode.upper(), 's;theme__day_theme_id': ids['day'],
                 's;theme__night_theme_id': ids['night']}, **prefs)


def fixture_theme():
    """The fixture extension as .flex bytes. Per seed, a day and a night component on the built-in
    Material You stylesheets with that seed's flags; `plain` and `font*`, the built-in day
    stylesheet as is and with an `@font` rule the root uses, naming the fixture face, a file of
    bytes that is no font, or a file the flex does not hold. Like image_theme(), the zip carries
    its directory entries: the app's unzip makes no parent directories."""
    manifest = json.loads((THEMES / 'org.florisboard.themes/extension.json').read_text())
    authors = manifest['themes'][0]['authors']
    manifest['meta'].update(id=EXT, title='Compose-free path test', description='Device check fixture')
    component = lambda ident, night, **extra: dict({'id': ident, 'label': ident, 'authors': authors, 'isNight': night}, **extra)
    themes, sheets = [], {}
    for name, (_, (style, contrast, spec)) in SEEDS.items():
        for mode in ('day', 'night'):
            flags = {'paletteStyle': style, 'contrastLevel': contrast, 'specVersion': spec}
            themes.append(component(f'my_{name}_{mode}', mode == 'night', materialYouFlags=flags))
            sheets[f'my_{name}_{mode}'] = (THEMES / f'org.florisboard.themes.my/stylesheets/floris_{mode}_my.json').read_text()
    day = json.loads((THEMES / 'org.florisboard.themes/stylesheets/floris_day.json').read_text())
    themes.append(component('plain', False))
    sheets['plain'] = json.dumps(day, indent=2)
    for name, face in FONT_CASES.items():
        ident = name.replace('-', '_')
        themes.append(component(ident, False))
        sheet = dict(day)
        sheet['@font `Fixture`'] = [{'src': f'uri(`flex:/fonts/{face}`)'}]
        if name == 'font':
            sheet['@font `Fixture`'].append({'src': 'uri(`flex:/fonts/bold.ttf`)', 'font-weight': 'bold'})
        sheet['root'] = {'font-family': '`Fixture`'}
        sheets[ident] = json.dumps(sheet, indent=2)
    manifest['themes'] = themes
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, 'w', zipfile.ZIP_DEFLATED) as flex:
        flex.writestr('extension.json', json.dumps(manifest, indent=2))
        flex.mkdir('stylesheets')
        flex.mkdir('fonts')
        flex.writestr('fonts/regular.ttf', (FONTS / 'fixture-script-regular-subset.ttf').read_bytes())
        flex.writestr('fonts/bold.ttf', (FONTS / 'fixture-script-bold-subset.ttf').read_bytes())
        flex.writestr('fonts/broken.ttf', b'not a font, ' * 64)
        for ident, sheet in sheets.items():
            flex.writestr(f'stylesheets/{ident}.json', sheet)
    return buffer.getvalue()


def press(device, *tests, wait=1.5):
    """Presses the first visible node, by its description, that the first of [tests] finding one
    accepts: an action, a panel's button, or a key such as the emoji key."""
    visible = nodes(device)
    found = next((hits for test in tests if (hits := [n for n in visible if test(n.get('description') or '')])), None)
    if not found:
        raise RuntimeError('No node for a fixed label')
    x, y, right, bottom = found[0]['bounds']
    agent('press', (x + right) // 2, (y + bottom) // 2)
    time.sleep(wait)


def named(label):
    return lambda description: description == label


def has(word):
    return lambda description: word in description.lower()


def shot(device, out, label, results, expect='views', grey_below=None, **extra):
    """Checks the composition and screenshots; with [grey_below], greys the screenshot from that
    row down before anything else reads it. Records the window's frame and the keyboard's top,
    the rows `compare` reads: above it is the probe's field, whose caret blinks."""
    ok, counts = check(out, expect, device.serial, label)
    path = dev['screenshot'](device, out, label)
    if grey_below is not None:
        rows = pixels(path)
        for y in range(grey_below, len(rows)):
            rows[y][:] = b'\x80\x80\x80\xff' * (len(rows[y]) // 4)
        path.write_bytes(png(rows, 4))
    window = diff['window'](device)
    pid = dev['pid_of'](device)
    results[label] = dict(counts, ok=ok and bool(pid), frame=window['frame'], content_top=window['content_top'],
                          alive=bool(pid), **extra)
    print(label, json.dumps({k: v for k, v in results[label].items() if k not in ('frame', 'content_top')}, sort_keys=True),
          flush=True)
    return ok


def panels(device, raw, out, results, mode):
    """Every panel with an icon in [mode]: keys, emoji, clipboard, overflow, editor, one-handed,
    subtype sheet, incognito watermark, GIF and its search."""
    def pose(prefs=None, **config):
        dev['pose'](device, raw, dict(config, prefs=theme(mode, **(prefs or {}))))

    steps = [
        ('keys', lambda: pose()),
        ('emoji', lambda: (pose(), press(device, named('Emoji')))),
        ('overflow', lambda: (pose(), press(device, named('More actions')))),
        ('editor', lambda: (pose(), press(device, named('More actions')), reorder(device))),
        ('one-handed', lambda: pose(one_handed='START')),
        ('sheet', lambda: (pose(prefs={'s;gestures__space_bar_long_press': 'SHOW_SUBTYPE_PICKER'}),
                           agent('longpress', *dev['point'](nodes(device), 'Space'), 1200), time.sleep(1.5))),
        ('incognito', lambda: pose(prefs={'s;suggestion__incognito_mode': 'FORCE_ON',
                                          's;keyboard__incognito_indicator': 'DISPLAY_BEHIND_KEYBOARD'})),
    ]
    for name, step in steps:
        attempt(f'{name}-{mode}', results, lambda: (step(), shot(device, out, f'{name}-{mode}', results, sheet=name == 'sheet')))
    attempt(f'clipboard-{mode}', results, lambda: clipboard(device, raw, out, results, mode, pose))
    attempt(f'gif-{mode}', results, lambda: gif(device, out, results, mode, pose))


def reorder(device):
    """Opens the actions editor from the overflow, scrolling its grid down to Reorder actions. The
    fling runs in the grid's left gutter, as panels.py's does, so no tile is armed."""
    for _ in range(5):
        try:
            return press(device, named('Reorder actions'))
        except RuntimeError:
            agent('swipe', 15, 2140, 15, 1760)
            time.sleep(1)
    raise RuntimeError('Reorder actions not reachable')


def clipboard(device, raw, out, results, mode, pose):
    """The clipboard panel, greyed from the keys' top down: its header, in the Smartbar's place,
    holds the icons checked, and nothing below it is kept."""
    pose()
    keys_top = min(n['bounds'][1] for n in nodes(device) if n['class'].endswith('Key'))
    press(device, named('Clipboard'))
    shot(device, out, f'clipboard-{mode}', results, grey_below=keys_top, greyed_from=keys_top)


def gif(device, out, results, mode, pose):
    """The media panel's GIF tab on the fixture build, then its search; see view-root-device.py."""
    pose()
    subprocess.run(['python3', '-I', str(HERE.parent / 'gif-fixture.py'), 'push', device.serial],
                   check=True, stdout=subprocess.DEVNULL)
    press(device, named('Emoji'))
    tab = next(n for n in nodes(device) if 'GIF' in ((n.get('description') or '') + (n.get('text') or '')).upper())
    x, y, right, bottom = tab['bounds']
    agent('press', (x + right) // 2, (y + bottom) // 2)
    time.sleep(2.5)
    shot(device, out, f'gif-{mode}', results)
    press(device, named('Search GIPHY'), has('search'), wait=2)
    shot(device, out, f'search-{mode}', results)


def material_you(device, raw, out, results):
    """Each seed's component, day and night, with that seed as the accent; and the built-in
    Material You theme on the system's accent."""
    for name, (argb, _) in SEEDS.items():
        for mode in ('day', 'night'):
            label = f'my-{name}-{mode}'
            attempt(label, results, lambda: (
                dev['pose'](device, raw, {'prefs': theme(mode, f'{EXT}:my_{name}_{mode}', **{'s;theme__accent_color': accent(argb)})}),
                shot(device, out, label, results)))
    attempt('my-system-day', results, lambda: (
        dev['pose'](device, raw, {'prefs': theme('day', 'org.florisboard.themes.my:floris_day_my',
                                                 **{'s;theme__accent_color': '0000000000000010'})}),
        shot(device, out, 'my-system-day', results)))


def fonts(device, raw, out, results):
    """The plain fixture theme before and after each font case: the fixture face must change the
    keys; a broken or missing face must leave them exactly as the plain theme draws them. A case is
    compared with a plain shot whose window has the same frame and keyboard top, since the top can
    move by a pixel between poses while SystemUI resizes its bars (E53); with none, it fails."""
    for name in ('plain', *FONT_CASES, 'plain-after'):
        ident = name.removesuffix('-after').replace('-', '_')
        attempt(name, results, lambda: (dev['pose'](device, raw, {'prefs': theme('day', f'{EXT}:{ident}')}),
                                        shot(device, out, name, results)))
    plains = [p for p in ('plain', 'plain-after') if p in results]
    for name in FONT_CASES:
        if name not in results:
            continue
        same = [p for p in plains if region(results[p]) == region(results[name])]
        if not same:
            results[name].update(differing_from_plain=None, ok=False, error='no plain shot with this window (E53)')
            print(name, 'has no plain shot with the same window: its keyboard top moved (E53)', flush=True)
            continue
        differing = window_diff(out / f'{same[0]}.png', out / f'{name}.png', region(results[same[0]]))
        want = differing > 1000 if name == 'font' else differing == 0
        results[name].update(differing_from_plain=differing, plain=same[0], ok=results[name]['ok'] and want)
        print(name, 'pixels differing from the plain theme:', differing, f'({same[0]})', flush=True)


def fallback(device, raw, out, results):
    """The Compose keyboard with the renderer switched off, day and night."""
    for mode in ('day', 'night'):
        label = f'fallback-{mode}'
        attempt(label, results, lambda: (
            dev['pose'](device, raw, {'prefs': dict(theme(mode), **{'b;keyboard__rust_renderer_prototype': False})}),
            shot(device, out, label, results, expect='compose')))


def attempt(label, results, run):
    """Runs one check; a failure is recorded under [label] and the run goes on."""
    try:
        run()
    except Exception as failure:  # one check's failure must not lose the others' results
        results[label] = dict(results.get(label, {}), ok=False, error=type(failure).__name__ + ': ' + str(failure)[:200])
        print(label, 'FAILED', results[label]['error'], flush=True)


def region(result):
    """The rows a check's screenshots are compared in: the input method window from the keyboard's
    top down, or from the window's top for the subtype sheet, which rises above the keyboard."""
    x0, y0, x1, y1 = result['frame']
    return x0, y0 if result.get('sheet') else max(y0, result['content_top'] or y0), x1, y1


def window_diff(a, b, box):
    """How many pixels inside [box] differ between screenshots [a] and [b]."""
    first, second = pixels(a), pixels(b)
    x0, y0, x1, y1 = box
    return sum(1 for y in range(y0, min(y1, len(first), len(second))) if first[y] != second[y]
               for x in range(x0 * 4, x1 * 4, 4) if first[y][x:x + 3] != second[y][x:x + 3])


CHECKS = {'panels-day': lambda d, r, o, res: panels(d, r, o, res, 'day'),
          'panels-night': lambda d, r, o, res: panels(d, r, o, res, 'night'),
          'material-you': material_you, 'fonts': fonts, 'fallback': fallback}


def run(args):
    device = dev['Device'](args.out)
    out = Path(args.out)
    raw, state = diff['saved'](out, device)
    results = {}
    try:
        device.mute()
        subprocess.run(['adb', '-s', device.serial, 'shell', f'run-as {PKG} sh -c "mkdir -p files/ime/theme && cat > {FLEX}"'],
                       input=fixture_theme(), check=True)
        for name in args.checks or CHECKS:
            CHECKS[name](device, raw, out, results)
    finally:
        device.adb('shell', 'run-as', PKG, 'rm', '-f', FLEX)
        diff['restore'](device, raw, state['system'], state['access'])
        gone = subprocess.run(['adb', '-s', device.serial, 'shell', 'run-as', PKG, 'test', '-e', FLEX]).returncode != 0
        print('fixture theme removed:', gone, flush=True)
        (out / 'results.json').write_text(json.dumps(results, indent=1, sort_keys=True) + '\n')
        for private in ('private-state.json', 'private-prefs.bin'):
            (out / private).unlink()
    failed = sorted(k for k, v in results.items() if not v['ok'])
    print(f'{len(results) - len(failed)} of {len(results)} checks passed; failed: {failed}')
    raise SystemExit(1 if failed else 0)


def compare(args):
    """Per label both runs have, the pixels differing inside the input method window."""
    first, second = (json.loads((Path(d) / 'results.json').read_text()) for d in (args.first, args.second))
    report = {}
    for label in sorted(set(first) & set(second)):
        a, b = first[label], second[label]
        if not (a.get('frame') and (a['frame'], a['content_top']) == (b.get('frame'), b.get('content_top'))):
            report[label] = {'differing': None, 'why': 'window frames differ or are missing'}
            print(label, 'frames differ or are missing', flush=True)
            continue
        report[label] = {'differing': window_diff(Path(args.first) / f'{label}.png', Path(args.second) / f'{label}.png', region(a))}
        print(label, report[label]['differing'], flush=True)
    only = sorted(set(first) ^ set(second))
    (Path(args.second) / 'compare.json').write_text(json.dumps({'labels': report, 'only_in_one': only}, indent=1) + '\n')
    print('identical:', sum(v['differing'] == 0 for v in report.values()), 'of', len(report), '; only in one run:', only)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest='command', required=True)
    runner = commands.add_parser('run')
    runner.add_argument('out')
    runner.add_argument('checks', nargs='*', help=f'any of {", ".join(CHECKS)} (default: all)')
    comparer = commands.add_parser('compare')
    comparer.add_argument('first')
    comparer.add_argument('second')
    args = parser.parse_args()
    unknown = set(getattr(args, 'checks', [])) - set(CHECKS)
    if unknown:
        parser.error(f'unknown checks: {sorted(unknown)}')
    (run if args.command == 'run' else compare)(args)


if __name__ == '__main__':
    main()
