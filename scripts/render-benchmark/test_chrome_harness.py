"""Safety and adversarial controls for the fixed chrome harness and semantic comparator."""
import copy
from pathlib import Path
import runpy
import unittest

api=runpy.run_path(str(Path(__file__).with_name('chrome.py')))
comparison=runpy.run_path(str(Path(__file__).with_name('compare-chrome.py')))


class HarnessTests(unittest.TestCase):
    def test_preferences_reject_unlisted_writes_before_reading_device(self):
        with self.assertRaises(ValueError):api['set_preferences'](None,{'unlisted_private_setting':'anything'})
    def test_empty_original_uses_two_latin_fixture_subtypes_without_mutating_original(self):
        original={};snapshot=copy.deepcopy(original);subtypes=api['subtypes'](original)
        self.assertEqual(snapshot,original)
        self.assertEqual(['en-US','en-GB'],[s['primaryLocale'] for s in subtypes])
        self.assertEqual([-1,8008],[s['id'] for s in subtypes])


Tests=comparison['Tests']
if __name__=='__main__':unittest.main()
