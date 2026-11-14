"""Check intent fidelity and failure/privacy boundaries without a device."""
import os
from pathlib import Path
import runpy
import subprocess
import unittest
from unittest.mock import patch

api = runpy.run_path(str(Path(__file__).with_name('agent-ui.py')))


class AgentTransportTests(unittest.TestCase):
    def test_settings_intent_keeps_browsable_category_and_real_ime(self):
        with patch.dict(os.environ, {'AGENT_DEVICE': '/bound/agent-device'}), patch.object(
                subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, '', '')) as call:
            api['launch'](('-W', '-a', 'android.intent.action.VIEW', '-c',
                'android.intent.category.BROWSABLE', '-d', 'ui://florisboard/settings/typing', '-p', 'fixture.package'))
        command = call.call_args.args[0]
        self.assertEqual('/bound/agent-device', command[0])
        self.assertIn('--no-test-ime', command)
        category = command.index('-c')
        self.assertEqual(['--launch-args', 'android.intent.category.BROWSABLE'], command[category + 1:category + 3])

    def test_success_does_not_return_ui_contents(self):
        with patch.dict(os.environ, {'AGENT_DEVICE': '/bound/agent-device'}), patch.object(
                subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'private-output', '')):
            self.assertEqual('', api['run']('press', 1, 2))

    def test_failed_device_action_has_no_automatic_retry_or_fallback(self):
        with patch.dict(os.environ, {'AGENT_DEVICE': '/bound/agent-device'}), patch.object(
                subprocess, 'run', return_value=subprocess.CompletedProcess([], 1, 'failed', '')) as call:
            with self.assertRaisesRegex(RuntimeError, 'failed'):
                api['run']('press', 1, 2)
        self.assertEqual(1, call.call_count)

    def test_unsupported_gesture_does_not_invent_a_fallback(self):
        with patch.object(subprocess, 'run') as call:
            with self.assertRaisesRegex(RuntimeError, 'cannot express'):
                api['touch'](('unsupported',))
        call.assert_not_called()


if __name__ == '__main__':
    unittest.main()
