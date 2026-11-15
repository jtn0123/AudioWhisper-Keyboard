#!/usr/bin/env python3
"""Empty-field prototype pictures through the T3-bound agent-device session only."""
import argparse
import importlib.util
from pathlib import Path
import sys
import time
sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('settings_device', Path(__file__).parent / 'render-benchmark/settings_ui_device.py')
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
DIALOGS = ['Hinted number row', 'Hinted symbols', 'Utility key action', 'Spacebar label', 'Capitalization behavior',
           'Font size multiplier', 'Incognito indicator', 'One-handed mode', 'One-handed mode keyboard width',
           'Landscape fullscreen input', 'Keyboard height', 'Key spacing', 'Bottom offset', 'Long key press delay']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('baseline', type=Path)
    parser.add_argument('--renderer', choices=['compose', 'rust', 'all'], default='all')
    parser.add_argument('--output', type=Path, default=Path('build/stage4-followup3/pictures'))
    parser.add_argument('--appearance', choices=['light', 'dark', 'font2', 'all'], default='all')
    args = parser.parse_args()
    base = args.output
    d = MODULE.SettingsDevice(base, args.baseline)
    assert d.adb('emu', 'avd', 'name').decode().splitlines()[0] == 'aw_keyboard_ssd'
    # Muted, and verified, before any app runs.
    for stream, volume in [(1, 0), (2, 0), (3, 0), (5, 0), (4, 1), (10, 1)]:
        d.adb('shell', 'cmd', 'audio', 'set-volume', str(stream), str(volume))
        assert d.adb('shell', 'cmd', 'audio', 'get-stream-volume', str(stream)).decode().strip().endswith(f'-> {volume}')
    try:
        d.launch('settings/home')
        for appearance in ('light', 'dark', 'font2') if args.appearance == 'all' else (args.appearance,):
            d.ui('settings', 'appearance', 'dark' if appearance == 'dark' else 'light')
            if appearance == 'font2':
                for preset in ('accessibility-extra-extra-large', 'accessibility-extra-large', 'accessibility-extra-extra-extra-large'):
                    d.ui('settings', 'text-size', preset)
                    if float(d.adb('shell', 'settings', 'get', 'system', 'font_scale')) == 2.0:
                        break
                else:
                    raise RuntimeError('No documented agent preset applies font scale 2.0')
            else:
                d.ui('settings', 'text-size', 'large')
            scale = float(d.adb('shell', 'settings', 'get', 'system', 'font_scale'))
            assert scale == (2.0 if appearance == 'font2' else 1.0)
            d.output = base / appearance
            d.output.mkdir(parents=True, exist_ok=True)
            d.save('environment.json', {'font_scale': scale, 'appearance': appearance, 'width_px': 1080})
            for renderer in (('compose', 'rust') if args.renderer == 'all' else (args.renderer,)):
                d.fixture()
                d.prototype(renderer)
                for route in ('home', 'keyboard'):
                    d.launch('settings/'+route)
                    d.wait(lambda: d.find('Keyboard') if route == 'home' else d.find('Number row'), 'Screen absent')
                    time.sleep(3.0)
                    d.empty_tree(renderer+'-'+route+'.json')
                    d.screenshot(renderer+'-'+route+'.png')
                d.ui('scroll', 'down', 1440)
                time.sleep(3.0)
                d.empty_tree(renderer+'-keyboard-lower.json')
                d.screenshot(renderer+'-keyboard-lower.png')
                # Enable the two dependent controls only for the dialog gallery.
                data = d.read_prefs()
                d.adb('shell', 'am', 'force-stop', MODULE.PKG)
                for key in ('keyboard__utility_key_enabled', 'keyboard__one_handed_mode_enabled'):
                    lines = [line for line in data.splitlines() if (';'+key+';').encode() not in line]
                    data = b'\n'.join(lines) + ('\nb;'+key+';true\n').encode()
                d.private_write(MODULE.PREFS, data)
                d.prototype(renderer)
                for i, label in enumerate(DIALOGS):
                    # Dialogs are visited in screen order; explicitly return to top first.
                    d.launch('settings/home')
                    d.launch('settings/keyboard')
                    d.ui('scroll', 'top')
                    d.press(label, scroll=True)
                    d.wait(lambda: d.find('Cancel'), 'Dialog absent')
                    time.sleep(3.0)
                    name = f'{renderer}-dialog-{i:02d}'
                    d.empty_tree(name+'.json')
                    d.screenshot(name+'.png')
                    d.press('Cancel')
                print(appearance, renderer, '17 pictures', flush=True)
    finally:
        d.ui('settings', 'text-size', 'large')
        d.ui('settings', 'appearance', 'light')
        d.output = base / 'cleanup'
        d.output.mkdir(parents=True, exist_ok=True)
        d.cleanup()


if __name__ == '__main__':
    main()
