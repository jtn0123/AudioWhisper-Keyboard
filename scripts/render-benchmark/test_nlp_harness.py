"""Adversarial candidate-strip comparator probes. Run python3 -I."""
import copy
import hashlib
from pathlib import Path
import runpy
import unittest

compare = runpy.run_path(str(Path(__file__).with_name('compare-nlp.py')))['compare']


class CandidateChecks(unittest.TestCase):
    def setUp(self):
        self.value = {'labelSha256': 'synthetic-label-digest', 'enabled': True, 'clickable': True,
            'clickAction': True, 'bounds': [270, 1562, 456, 1675], 'labelBounds': [300, 1590, 426, 1647]}
    def test_good_strip(self): self.assertEqual([], compare([self.value], [self.value], 'typing'))
    def test_wrong_label(self):
        other = copy.deepcopy(self.value); other['labelSha256'] = 'wrong'
        self.assertTrue(compare([self.value], [other], 'typing'))
    def test_missing_click(self):
        other = copy.deepcopy(self.value); other['clickAction'] = False
        self.assertTrue(compare([self.value], [other], 'typing'))
    def test_displaced_target(self):
        other = copy.deepcopy(self.value); other['bounds'][0] += 20; other['bounds'][2] += 20
        self.assertTrue(compare([self.value], [other], 'typing'))
    def test_password_candidates(self): self.assertTrue(compare([self.value], [self.value], 'password'))
    def test_password_actions_are_controls(self):
        control = copy.deepcopy(self.value); control['labelSha256'] = hashlib.sha256(b'Undo').hexdigest()
        self.assertEqual([], compare([control], [control], 'password'))


if __name__ == '__main__': unittest.main()
