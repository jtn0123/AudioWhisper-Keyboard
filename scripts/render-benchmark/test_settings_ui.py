"""Settings runner regressions: nested Compose targets and stock-IME ownership."""
from pathlib import Path
import runpy
import unittest

api = runpy.run_path(str(Path(__file__).with_name('settings_ui_device.py')))


class SettingsUiTests(unittest.TestCase):
    def test_nested_compose_label_uses_specific_semantic_selector(self):
        device = object.__new__(api['SettingsDevice'])
        node = {'ref': 'e27', 'kind': 'text', 'label': 'Cancel'}
        device.find = lambda label: node
        commands = []

        def ui(*args):
            commands.append(args)
            if args[1].startswith('@'):
                raise RuntimeError('Ref has no parent-owned touch point outside its interactive descendants')

        device.ui = ui
        device.press('Cancel')
        self.assertEqual([('press', 'role=text label="Cancel"')], commands)

    def test_native_combined_label_keeps_its_role_and_complete_label(self):
        device = object.__new__(api['SettingsDevice'])
        device.find = lambda label: {'ref': 'e4', 'kind': 'switch', 'label': 'Number row, Show a number row'}
        commands = []
        device.ui = lambda *args: commands.append(args)
        device.press('Number row')
        self.assertEqual([('press', 'role=switch label="Number row, Show a number row"')], commands)

    def test_stock_keyboard_must_be_system_installed_and_not_our_package(self):
        device = object.__new__(api['SettingsDevice'])
        calls = []
        def adb(*args):
            calls.append(args)
            if args == ('shell', 'ime', 'list', '-a', '-s'):
                return b'com.jtn0123.audiowhisper.keyboard/.Ime\ncom.agent.helper/.Ime\ncom.android.inputmethod.latin/.LatinIME\n'
            if args == ('shell', 'pm', 'list', 'packages', '-s'):
                return b'package:com.android.inputmethod.latin\n'
            if args == ('shell', 'settings', 'get', 'secure', 'default_input_method'):
                return b'com.android.inputmethod.latin/.LatinIME\n'
            return b''
        device.adb = adb
        self.assertEqual('com.android.inputmethod.latin/.LatinIME', device.stock_ime())
        self.assertIn(('shell', 'ime', 'set', 'com.android.inputmethod.latin/.LatinIME'), calls)

    def test_absent_system_keyboard_refuses_without_switching(self):
        device = object.__new__(api['SettingsDevice'])
        calls = []
        def adb(*args):
            calls.append(args)
            return b'com.agent.helper/.Ime\n' if 'ime' in args else b'package:com.agent.helper\n'
        device.adb = adb
        with self.assertRaisesRegex(RuntimeError, 'No stock system keyboard'):
            device.stock_ime()
        self.assertFalse(any('set' in c for c in calls))


if __name__ == '__main__':
    unittest.main()
