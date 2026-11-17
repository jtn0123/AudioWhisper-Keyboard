"""Semantic settings actions on the Rust settings, with a receipt of the complete stored file.

It compared Compose and Rust until the Compose screens were removed (docs/compose-removal.md).
"""
import argparse
import hashlib
from pathlib import Path
import runpy

api = runpy.run_path(str(Path(__file__).with_name('settings_ui_device.py')))
SettingsDevice = api['SettingsDevice']


def actions(d):
    d.launch('settings/keyboard')
    d.press('Number row')
    d.wait(lambda: d.prefs().get('keyboard__number_row') == 'true', 'Number-row write absent')
    d.press('Spacebar label')
    d.press('No label')
    d.press('OK')
    d.press('Font size multiplier')
    d.slider_minimum()
    d.press('Cancel')
    if d.prefs().get('keyboard__font_size_multiplier_portrait', '100') != '100':
        raise RuntimeError('Cancelled slider changed the stored value')
    d.press('Font size multiplier')
    d.slider_minimum()
    d.press('OK')
    d.wait(lambda: d.prefs().get('keyboard__font_size_multiplier_portrait') == '50', 'Slider minimum absent')
    d.press('Popup Visibility', scroll=True)
    d.ui('back')
    # The IME-picker control is the bottom-right button.
    nodes = [n for n in d.app_nodes() if n.get('kind') == 'button' and n['rect']['y'] > 1800]
    if not nodes:
        raise RuntimeError('IME-picker control absent')
    picker = max(nodes, key=lambda n: n['rect']['x'])
    d.ui('press', '@' + picker['ref'])
    d.ui('back')
    stock = d.stock_ime()
    d.save('stock-ime.json', {'component': stock})
    try:
        d.launch('settings/home')
        field = next(n for n in d.app_nodes() if n.get('editable'))
        d.ui('press', '@' + field['ref'])
        # Fixed synthetic fixture only. Neither UI output nor the filled tree is persisted.
        value = 'settingsfixture'
        d.ui('type', value)
        expected = {'utf16_units': len(value), 'sha256': hashlib.sha256(value.encode()).hexdigest()}
        digest = d.field_digest()
        if digest != expected:
            raise RuntimeError('Preview field digest differs')
        field = next(n for n in d.app_nodes() if n.get('editable'))
        d.ui('fill', '@' + field['ref'], '')
        if d.field_digest()['utf16_units'] != 0:
            raise RuntimeError('Preview field did not clear')
        d.ui('back')
    finally:
        d.restore_ime()
    return digest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('baseline', type=Path)
    args = parser.parse_args()
    d = SettingsDevice(args.output, args.baseline)
    try:
        d.fixture()
        d.prototype('rust')
        digest = actions(d)
        d.save('result.json', {'field': digest, 'preferences': d.file_receipt('rust')})
    finally:
        d.cleanup()


if __name__ == '__main__':
    main()
