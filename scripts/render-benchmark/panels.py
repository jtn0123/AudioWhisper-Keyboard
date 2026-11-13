"""Panel fixtures over the serial-scoped adb harness; screenshots use agent-device only.

Only public, fixed fixture text enters the disposable probe. Results contain digests and
semantic panel trees, never editor contents. Pass the original preference snapshot for cleanup.
"""
import argparse
import hashlib
import json
from pathlib import Path
import runpy
import shlex
import subprocess
import time

api = runpy.run_path(str(Path(__file__).with_name('a11y_device.py')))
SERVICE, PKG, PROBE = (api[k] for k in ('SERVICE', 'PKG', 'PROBE'))


class Device(api['Device']):
    def tap(self, x, y):
        super().tap(x, y)
        # Let a panel's disposal/recreation finish before the next action. Otherwise a
        # close+open within one Compose frame retains the old lazy-grid scroll position.
        time.sleep(.25)

    def reset(self, no_suggestions=False, field='text'):
        super().reset(no_suggestions, field)
        time.sleep(.3)


def named(nodes, label):
    for index, node in enumerate(nodes):
        if node['visible'] and label in (node.get('text'), node.get('description')):
            target = node
            if node.get('clickable') or node.get('class') in ('android.widget.Button', 'android.widget.Switch', 'android.widget.TextView'):
                return target
            parent = node['parent']
            while parent >= 0:
                ancestor = nodes[parent]
                bounds = ancestor['bounds']
                width, height = bounds[2] - bounds[0], bounds[3] - bounds[1]
                if (ancestor.get('clickable') and width <= 400) or (150 <= width <= 300 and 0 < height <= 300):
                    target = ancestor
                    break
                parent = ancestor['parent']
            return target
    raise RuntimeError('Expected fixed panel target missing')


def center(node):
    x, y, right, bottom = node['bounds']
    return ((x + right) // 2, (y + bottom) // 2)


def tap_name(device, label, nodes=None):
    device.tap(*center(named(nodes or device.tree()['nodes'], label)))


def observer_off(device):
    device.shell('settings', 'put', 'secure', 'enabled_accessibility_services', '')
    device.shell('settings', 'put', 'secure', 'accessibility_enabled', 0)
    device.wait(lambda: not any('label=Renderer probe' in line and 'Bound services:' in line
                               for line in device.shell('dumpsys', 'accessibility').splitlines()),
                'Observer did not unbind')
    time.sleep(.5)


def scroll(device, down=True):
    # The content gutter avoids arming an editing key before the scroll gesture wins.
    device.shell('input', 'swipe', 15, 2140 if down else 1760, 15, 1760 if down else 2140, 500)
    time.sleep(.3)


def overflow(device):
    data = device.tree()
    if not any('More actions' in (n.get('text'), n.get('description')) for n in data.get('nodes', [])):
        device.tap(1018, 1618)
        device.capture('expanded-actions', lambda ns: any('More actions' in (n.get('text'), n.get('description')) for n in ns))
    tap_name(device, 'More actions')
    return device.capture('overflow')


def edit(device):
    for _ in range(5):
        data = device.tree()
        try:
            tap_name(device, 'Reorder actions', data['nodes'])
            return device.capture('edit')
        except RuntimeError:
            scroll(device)
    raise RuntimeError('Customize action not reachable')


def emoji(device, category):
    tap_name(device, 'Emoji')
    device.wait(lambda: any(n.get('text') == 'ABC' or n.get('description') == 'ABC' for n in device.tree().get('nodes', [])), 'Emoji palette absent')
    # Tab cells are equal-weight across the panel, as in Compose PrimaryTabRow.
    device.tap(int((category + .5) * 108), 1610)
    return device.capture('emoji-category-' + str(category))


def screenshot(device, agent, name):
    path = device.output.resolve() / (name + '.png')
    result = subprocess.check_output([agent, 'screenshot', str(path)], text=True)
    path.with_suffix('.capture.json').write_text(result)


def digest(device, expected, name):
    expected_units = len(expected.encode('utf-16-le')) // 2
    result = device.digest()
    device.save(name + '-actual.json', result)
    result = device.wait(lambda: (r if (r := device.digest())['utf16_units'] == expected_units else None), 'Fixture commit did not settle')
    wanted = hashlib.sha256(expected.encode()).hexdigest()
    assert result['sha256'] == wanted and result['utf16_units'] == len(expected.encode('utf-16-le')) // 2, name
    device.save(name + '-digest.json', result)
    return result


def checkpoint(device, name, units, start, end):
    actual = device.digest()
    device.save(name + '-checkpoint.json', actual)
    device.wait(lambda: (d if (d := device.digest())['utf16_units'] == units and d['selection_start'] == start and d['selection_end'] == end else None), 'Editor action checkpoint failed: ' + name)
    # The probe publishes its cursor before the IME receives onUpdateSelection.
    time.sleep(.5)


def restore_preferences(device, original, keys):
    """Restore exact serialized values and absence after stopping the datastore owner."""
    raw = device.adb('exec-out', 'run-as', PKG, 'cat', api['PREFS'])
    rows = raw.splitlines()
    originals = {k: v for k, v in json.loads(Path(original).read_text()).items() if k in keys}
    types = {parts[1]: parts[0] for row in rows if len(parts := row.split(';', 2)) == 3}
    result = [row for row in rows if len(parts := row.split(';', 2)) != 3 or parts[1] not in keys]
    for key, value in originals.items():
        result.append(types.get(key, 'str') + ';' + key + ';' + value)
    device.shell('am', 'force-stop', PKG)
    # stdin is data, not shell code. Preserve the existing preference file's permissions.
    subprocess.run(['adb', '-s', device.serial, 'shell', 'run-as ' + PKG + ' sh -c ' + shlex.quote('cat > ' + api['PREFS'])], input=('\n'.join(result) + '\n').encode(), check=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('renderer', choices=('compose', 'rust'))
    parser.add_argument('output')
    parser.add_argument('--agent')
    parser.add_argument('--visuals', action='store_true')
    args = parser.parse_args()
    device = Device(args.output)
    original = device.original_accessibility()
    original_panel_prefs = device.output / 'original-panel-preferences.json'
    device.save(original_panel_prefs.name, device.prefs())
    capture_preferences = device.capture_preferences()
    try:
        device.renderer(args.renderer)
        device.probe(True)
        device.services(SERVICE)
        device.capture("preflight", lambda ns: any("Emoji" in (n.get("text"), n.get("description")) for n in ns))
        # Empty-editor captures are taken before any fixture input.
        data = emoji(device, 1)
        if args.visuals:
            screenshot(device, args.agent, 'smileys')
        positions = [center(named(data['nodes'], value)) for value in ('😀', '😃', '😄', '😁')]
        for position in positions:
            device.tap(*position)
        device.tap(270, 1610)
        time.sleep(.6)
        device.hold(60, 1735, 700)
        device.tap(55, 1618)
        digest(device, '😀😃😄😁👋🏻', 'five-emoji')
        device.reset(True)
        device.capture('keyboard-before-recents')
        data = emoji(device, 0)
        order = [n.get('text') or n.get('description') for n in data['nodes'] if n['visible']]
        seen = [value for value in order if value in ('😀', '😃', '😄', '😁', '👋🏻')]
        assert seen == ['👋🏻', '😁', '😄', '😃', '😀'], 'Recent history order differs'
        device.save('recents-verdict.json', {'matches': True, 'items': len(seen)})
        tap_name(device, 'ABC')
        top = overflow(device)
        for _ in range(3):
            scroll(device)
        bottom = device.capture('editing-buttons')
        # Record public geometry while empty, and stop the observer for typed input.
        # A fresh probe and gutter scrolling keep old requests and editing-key holds
        # out of navigation. No tree or geometry query runs while text is selected.
        points = {label: center(named(data['nodes'], label)) for data, labels in
                  ((top, ('More actions', 'Arrow left', 'Arrow right')),
                   (bottom, ('Select all', 'Copy', 'Paste'))) for label in labels}
        observer_off(device)
        device.tap(*points['More actions'])
        device.shell('input', 'text', 'abcd')
        device.tap(*points['More actions'])
        device.tap(*points['Arrow left'])
        checkpoint(device, 'arrow-left', 4, 3, 3)
        for _ in range(3):
            scroll(device)
        device.tap(*points['Select all'])
        checkpoint(device, 'select-all', 4, 0, 4)
        device.tap(*points['Copy'])
        checkpoint(device, 'copy', 4, 4, 4)
        device.tap(*points['More actions'])
        device.tap(*points['Arrow right'])
        checkpoint(device, 'arrow-right', 4, 4, 4)
        for _ in range(3):
            scroll(device)
        device.tap(*points['Paste'])
        digest(device, 'abcdabcd', 'editing')
        device.reset(True)
        device.services(SERVICE)
        device.capture('after-editing-reset')
        overflow(device)
        editor = edit(device)
        if args.visuals:
            screenshot(device, args.agent, 'edit')
        source = center(named(editor['nodes'], 'Undo'))
        destination = center(named(editor['nodes'], 'Redo'))
        # A stationary long press followed by moves is required; a plain swipe scrolls.
        device.shell('input', 'motionevent', 'DOWN', *source)
        time.sleep(.7)
        for step in range(1, 9):
            x = round(source[0] + (destination[0] - source[0]) * step / 8)
            y = round(source[1] + (destination[1] - source[1]) * step / 8)
            device.shell('input', 'motionevent', 'MOVE', x, y)
        device.shell('input', 'motionevent', 'UP', *destination)
        # Back is the editor header's leading 48dp target in both implementations.
        header = named(editor['nodes'], 'Customize action order')['bounds']
        device.tap(63, (header[1] + header[3]) // 2)
        device.wait(lambda: 'smartbar__action_arrangement' in device.prefs(), 'Arrangement did not persist')
        stored = device.prefs()['smartbar__action_arrangement']
        device.save('arrangement.json', {'serialized': stored, 'sha256': hashlib.sha256(stored.encode()).hexdigest()})

    finally:
        device.restore_capture_preferences(capture_preferences)
        restore_preferences(device, original_panel_prefs, {'emoji__history_data', 'smartbar__action_arrangement'})
        device.restore_accessibility(original)


if __name__ == '__main__':
    main()
