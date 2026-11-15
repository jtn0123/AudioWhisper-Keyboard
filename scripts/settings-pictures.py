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
# Group 1: (route, a label the screen shows at its top, lower pages, dialog rows in screen order).
GROUP_ONE = [
    ('keyboard/input-feedback', 'Enable audio feedback', 1, ['Enable audio feedback', 'Sound volume for input events',
        'Enable haptic feedback', 'Vibration mode', 'Vibration duration', 'Vibration strength']),
    ('smartbar', 'Enable Smartbar', 0, ['Layout', 'Suggestions display mode', 'Action row placement']),
    ('gestures', 'Enable glide typing', 2, ['Trail fade duration', 'Preview refresh delay', 'Swipe velocity threshold',
        'Swipe distance threshold']),
    ('clipboard', 'Use internal clipboard', 2, ['Sync from system clipboard', 'Sync to system clipboard',
        'Limit clipboard suggestions to', 'Number of grid columns', 'Clean up old items after',
        'Auto clean sensitive items after', 'Max history size']),
    ('media', 'Preferred emoji skin tone', 1, ['Preferred emoji skin tone', 'Update strategy (Pinned)',
        'Update strategy (Recent)', 'Maximum items to keep', 'Reset pinned emojis', 'Reset recent emojis',
        'Trigger type', 'Minimum query length', 'Maximum candidate count']),
]
# The swipe lists are enabled only while glide typing is off.
SWIPE_DIALOGS = ['Swipe up', 'Swipe down', 'Swipe left', 'Swipe right', 'Space bar swipe up', 'Space bar swipe left',
                 'Space bar swipe right', 'Space bar long press', 'Delete key swipe left', 'Delete key long press']
# Every master on, so every conditional dialog row is enabled for its picture.
MASTERS = {'input_feedback__audio_enabled': 'true', 'input_feedback__haptic_enabled': 'true',
           'smartbar__enabled': 'true', 'glide__enabled': 'true', 'glide__show_trail': 'true', 'glide__show_preview': 'true',
           'clipboard__use_internal_clipboard': 'true', 'clipboard__suggestion_enabled': 'true',
           'clipboard__history_enabled': 'true', 'clipboard__history_auto_clean_old_enabled': 'true',
           'clipboard__history_auto_clean_sensitive_enabled': 'true', 'clipboard__history_size_limit_enabled': 'true',
           'emoji__history_enabled': 'true', 'emoji__suggestion_enabled': 'true'}
ENUMS = {'input_feedback__haptic_vibration_mode': 'USE_VIBRATOR_DIRECTLY', 'smartbar__layout': 'SUGGESTIONS_ACTIONS_EXTENDED'}


def with_prefs(data, booleans, strings):
    keys = [*booleans, *strings]
    lines = [line for line in data.splitlines() if not any((';'+key+';').encode() in line for key in keys)]
    lines += [f'b;{k};{v}'.encode() for k, v in booleans.items()] + [f's;{k};{v}'.encode() for k, v in strings.items()]
    return b'\n'.join(lines) + b'\n'


def picture(d, name):
    time.sleep(3.0)
    d.empty_tree(name+'.json')
    d.screenshot(name+'.png')


def group_one(d, renderer):
    """Each group 1 screen from its top, then every dialog row with the masters it needs on."""
    d.fixture()
    d.prototype(renderer)
    for route, ready, pages, _ in GROUP_ONE:
        name = renderer+'-'+route.replace('/', '-')
        d.launch('settings/'+route)
        d.wait(lambda: d.find(ready), 'Screen absent')
        picture(d, name)
        for page in range(pages):
            d.ui('scroll', 'down', 1440)
            picture(d, f'{name}-lower{page+1}')
    skipped = []
    for glide, dialogs in (('true', [(r, label) for r, _, _, rows in GROUP_ONE for label in rows]),
                           ('false', [('gestures', label) for label in SWIPE_DIALOGS])):
        data = with_prefs(d.read_prefs(), {**MASTERS, 'glide__enabled': glide}, ENUMS)
        d.adb('shell', 'am', 'force-stop', MODULE.PKG)
        d.private_write(MODULE.PREFS, data)
        d.prototype(renderer)
        for route, label in dialogs:
            d.launch('settings/home')
            d.launch('settings/'+route)
            d.ui('scroll', 'top')
            node = None
            for _ in range(12):
                node = d.find(label)
                if node:
                    break
                d.ui('scroll', 'down', 480, '--duration-ms', 1200)
            # Duration and strength follow the vibrator facts, as InputFeedbackScreen does.
            if node is None or node.get('enabled') is False:
                skipped.append(label)
                continue
            d.press(label)
            d.wait(lambda: d.find('Cancel') or d.find('No'), 'Dialog absent')
            picture(d, f"{renderer}-dialog-{route.replace('/', '-')}-{label.lower().replace(' ', '-').replace('(', '').replace(')', '')}")
            d.press('Cancel' if d.find('Cancel') else 'No')
    d.save(renderer+'-group-one-skipped.json', skipped)


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
                group_one(d, renderer)
                print(appearance, renderer, 'group 1 pictures', flush=True)
    finally:
        d.ui('settings', 'text-size', 'large')
        d.ui('settings', 'appearance', 'light')
        d.output = base / 'cleanup'
        d.output.mkdir(parents=True, exist_ok=True)
        d.cleanup()


if __name__ == '__main__':
    main()
