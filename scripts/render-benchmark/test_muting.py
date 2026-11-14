"""Check that muting sets each stream through AudioService and refuses an unverified level."""
import os
from pathlib import Path
import runpy
import tempfile
import unittest
from unittest.mock import patch

api = runpy.run_path(str(Path(__file__).with_name('a11y_device.py')))
TARGETS = {1: 0, 2: 0, 3: 0, 5: 0, 10: 1, 4: 1}


class FakeAudio:
    """Records shell calls; `stuck` streams ignore set-volume, as media_session did."""

    def __init__(self, stuck=()):
        self.levels = {stream: 5 for stream in TARGETS}
        self.stuck = set(stuck)
        self.calls = []

    def __call__(self, *args):
        args = tuple(map(str, args))
        self.calls.append(args)
        if args[:3] == ('cmd', 'audio', 'set-volume'):
            if int(args[3]) not in self.stuck:
                self.levels[int(args[3])] = int(args[4])
            return f'calling AudioManager.setStreamVolume({args[3]}, {args[4]}, 0)\n'
        if args[:3] == ('cmd', 'audio', 'get-stream-volume'):
            return f'AudioManager.getStreamVolume({args[3]}) -> {self.levels[int(args[3])]}\n'
        return ''


class MutingTests(unittest.TestCase):
    def device(self, audio, output):
        with patch.dict(os.environ, {'ANDROID_SERIAL': 'emulator-5556'}):
            device = api['Device'](output)
        device.shell = audio
        return device

    def test_every_stream_reaches_its_minimum_through_audio_service(self):
        audio = FakeAudio()
        with tempfile.TemporaryDirectory() as output:
            self.device(audio, output).mute()
            self.assertTrue((Path(output) / 'muting.json').exists())
        self.assertEqual(audio.levels, TARGETS)
        self.assertFalse([call for call in audio.calls if 'media_session' in call])

    def test_a_level_that_does_not_change_stops_the_run(self):
        audio = FakeAudio(stuck={3})
        with tempfile.TemporaryDirectory() as output:
            with self.assertRaisesRegex(RuntimeError, 'Stream 3 is at 5, not 0'):
                self.device(audio, output).mute()
            self.assertFalse((Path(output) / 'muting.json').exists())


if __name__ == '__main__':
    unittest.main()
