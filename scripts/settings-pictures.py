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
# Group 2, the same way. Spelling's Languages list is enabled only while FlorisBoard is the
# system spell checker; a skipped entry says so.
GROUP_TWO = [
    ('typing', 'Display suggestions', 2, ['Incognito mode', 'Forget learned words', 'Languages']),
    ('dictionary', 'Enable system user dictionary', 0, []),
    ('other/physical-keyboard', 'Show on-screen keyboard', 0, []),
    ('about', 'Version', 0, []),
    ('dictation', 'Remove filler words', 1, []),
]
# Suggestions on for its dependent rows, auto-space on for the feedback card.
GROUP_TWO_MASTERS = {'suggestion__enabled': 'true', 'correction__auto_space_punctuation': 'true'}
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


def without(data, keys):
    return b'\n'.join(line for line in data.splitlines() if not any((';'+key+';').encode() in line for key in keys)) + b'\n'


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
            # Duration and strength follow the vibrator facts.
            if node is None or node.get('enabled') is False:
                skipped.append(label)
                continue
            d.press(label)
            d.wait(lambda: d.find('Cancel') or d.find('No'), 'Dialog absent')
            picture(d, f"{renderer}-dialog-{route.replace('/', '-')}-{label.lower().replace(' ', '-').replace('(', '').replace(')', '')}")
            d.press('Cancel' if d.find('Cancel') else 'No')
    d.save(renderer+'-group-one-skipped.json', skipped)


def group_two(d, renderer):
    """Each group 2 screen from its top, then its dialogs. The vocabulary is cleared first, so
    the field shows only its placeholder and no picture holds what the user wrote."""
    d.fixture()
    data = with_prefs(without(d.read_prefs(), ['dictation__vocabulary']), GROUP_TWO_MASTERS, {})
    d.adb('shell', 'am', 'force-stop', MODULE.PKG)
    d.private_write(MODULE.PREFS, data)
    d.prototype(renderer)
    # Languages is enabled only while this app is the system spell checker. Compose reports
    # enabled on the row's text, not on the disabled row, so the fact decides for both.
    spell = d.adb('shell', 'settings', 'get', 'secure', 'selected_spell_checker').decode().strip()
    skipped = [] if spell.startswith(MODULE.PKG + '/') else ['Languages']
    for route, ready, pages, dialogs in GROUP_TWO:
        name = renderer+'-'+route.replace('/', '-')
        d.launch('settings/home')
        d.launch('settings/'+route)
        d.wait(lambda: d.find(ready), 'Screen absent')
        picture(d, name)
        for page in range(pages):
            d.ui('scroll', 'down', 1440)
            picture(d, f'{name}-lower{page+1}')
        for label in dialogs:
            if label in skipped:
                continue
            d.launch('settings/home')
            d.launch('settings/'+route)
            d.ui('scroll', 'top')
            node = None
            for _ in range(12):
                node = d.find(label)
                if node:
                    break
                d.ui('scroll', 'down', 480, '--duration-ms', 1200)
            if node is None or node.get('enabled') is False:
                skipped.append(label)
                continue
            d.press(label)
            d.wait(lambda: d.find('Cancel'), 'Dialog absent')
            picture(d, f"{renderer}-dialog-{route.replace('/', '-')}-{label.lower().replace(' ', '-')}")
            d.press('Cancel')
    d.save(renderer+'-group-two-skipped.json', skipped)


# Step 4 (docs/stage4-other-theme.md): every row enabled (Follow time), the stored colours and
# times at their defaults, so the pictures hold no personal choice.
# Stored strings are JSON: quoted.
OTHER_THEME_PREFS = {'theme__mode': '"FOLLOW_TIME"', 'other__settings_theme': '"AUTO"', 'other__accent_color': '"0000000000000010"',
                     'theme__accent_color': '"0000000000000010"', 'theme__sunrise_time': '"06:00:00.000"',
                     'theme__sunset_time': '"18:00:00.000"', 'theme__day_theme_id': '"org.florisboard.themes:floris_day"',
                     'theme__night_theme_id': '"org.florisboard.themes:floris_night"'}
# (route, a label at its top, [(dialog row, [buttons pressed in turn, a picture after each])]).
OTHER_THEME = [
    ('other', 'Settings theme', [('Settings theme', []), ('Settings accent color', ['Custom'])]),
    ('theme', 'Theme mode', [('Theme mode', []), ('Sunrise time', ['Switch time input to keyboard input']),
                             ('Accent color (Material You themes)', ['Custom'])]),
]


def other_theme(d, renderer):
    """Other and Theme from their tops, then each dialog and each of its other layouts."""
    d.fixture()
    data = with_prefs(d.read_prefs(), {}, OTHER_THEME_PREFS)
    d.adb('shell', 'am', 'force-stop', MODULE.PKG)
    d.private_write(MODULE.PREFS, data)
    d.prototype(renderer)
    for route, ready, dialogs in OTHER_THEME:
        d.launch('settings/home')
        d.launch('settings/'+route)
        d.wait(lambda: d.find(ready), 'Screen absent')
        picture(d, renderer+'-'+route)
        for label, steps in dialogs:
            d.launch('settings/home')
            d.launch('settings/'+route)
            d.wait(lambda: d.find(label), 'Row absent')
            d.press(label)
            d.wait(lambda: d.find('Cancel'), 'Dialog absent')
            name = f"{renderer}-dialog-{route}-{label.lower().replace(' ', '-').replace('(', '').replace(')', '')}"
            picture(d, name)
            for step, button in enumerate(steps):
                d.press(button)
                # The time dialog's keyboard entry holds the time in text fields, and a tree with
                # filled fields is never written, so that step keeps only its screenshot.
                if button.endswith('keyboard input'):
                    time.sleep(3.0)
                    d.screenshot(f'{name}-{step+1}.png')
                else:
                    picture(d, f'{name}-{step+1}')
            d.press('Cancel')


def licences(d, renderer):
    """The third-party licences list at its top and a page down, the first library's licence
    dialog, and the project licence screen, which Rust draws since compose-removal step 8."""
    d.fixture()
    d.prototype(renderer)
    d.launch('settings/home')
    d.launch('settings/about/third-party-licenses')
    d.wait(lambda: d.find('Activity'), 'Screen absent')
    picture(d, renderer+'-licences')
    d.ui('scroll', 'down', 1440)
    picture(d, renderer+'-licences-lower1')
    # Back to the top by opening the screen again: agent-device reads Compose's LazyColumn (a
    # View with CollectionInfo, not a ScrollView) as already at the top, so `scroll top` stays.
    d.launch('settings/home')
    d.launch('settings/about/third-party-licenses')
    d.wait(lambda: d.find('Activity'), 'Screen absent')
    d.press('Activity')
    d.wait(lambda: d.find('OK'), 'Dialog absent')
    picture(d, renderer+'-dialog-licences-activity')
    d.press('OK')
    d.launch('settings/home')
    d.launch('settings/about/project-license')
    # By its title: the licence is one Text node, the whole file, indented as the file is.
    d.wait(lambda: d.find('Project license'), 'Screen absent')
    picture(d, renderer+'-project-license')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('baseline', type=Path)
    # Rust only: the Compose settings screens were removed (docs/compose-removal.md).
    parser.add_argument('--renderer', choices=['rust'], default='rust')
    parser.add_argument('--output', type=Path, default=Path('build/stage4-followup3/pictures'))
    parser.add_argument('--appearance', choices=['light', 'dark', 'font2', 'all'], default='all')
    parser.add_argument('--sections', nargs='+', choices=['keyboard', 'group-one', 'group-two', 'licences', 'other-theme'],
                        default=['keyboard', 'group-one', 'group-two', 'licences', 'other-theme'])
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
            for renderer in (args.renderer,):
                if 'keyboard' in args.sections:
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
                if 'group-one' in args.sections:
                    group_one(d, renderer)
                    print(appearance, renderer, 'group 1 pictures', flush=True)
                if 'group-two' in args.sections:
                    group_two(d, renderer)
                    print(appearance, renderer, 'group 2 pictures', flush=True)
                if 'licences' in args.sections:
                    licences(d, renderer)
                    print(appearance, renderer, 'licences pictures', flush=True)
                if 'other-theme' in args.sections:
                    other_theme(d, renderer)
                    print(appearance, renderer, 'other and theme pictures', flush=True)
    finally:
        d.ui('settings', 'text-size', 'large')
        d.ui('settings', 'appearance', 'light')
        d.output = base / 'cleanup'
        d.output.mkdir(parents=True, exist_ok=True)
        d.cleanup()
        # Leave no agent session behind: its helper holds UiAutomation, and the next
        # instrumented run fails to start ("UiAutomationService ... already registered").
        d.ui('close')


if __name__ == '__main__':
    main()
