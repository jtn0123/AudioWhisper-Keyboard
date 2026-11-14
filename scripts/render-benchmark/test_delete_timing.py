"""Schedule oracle controls: jitter, exact delivered count and the one-ms interval mutation."""
from pathlib import Path
import runpy
import unittest

oracle = runpy.run_path(str(Path(__file__).with_name('delete-timing.py')))


def trace(interval=50):
    # Real delays reschedule after work, never from a hypothetical exact two-second grid.
    return '\n'.join([
        'down down_ms=100 at_ms=100',
        'hold down_ms=100 at_ms=102 due_ms=402 reference_due_ms=402',
        'attempt down_ms=100 at_ms=409',
        'tick down_ms=100 at_ms=410',
        f'repeat down_ms=100 at_ms=415 due_ms={415 + interval} reference_due_ms=465',
        'attempt down_ms=100 at_ms=476',
        'tick down_ms=100 at_ms=477',
        f'repeat down_ms=100 at_ms=480 due_ms={480 + interval} reference_due_ms=530',
        'up down_ms=100 at_ms=500'])


class DeleteTimingTest(unittest.TestCase):
    def test_jitter_has_exactly_two_actual_repeat_opportunities(self):
        self.assertEqual(2, oracle['verify'](trace(), 1088, 1086)['expected_deletes'])

    def test_one_missing_or_extra_delete_fails(self):
        for after in (1085, 1087):
            with self.assertRaisesRegex(AssertionError, 'Delivered deletes'):
                oracle['verify'](trace(), 1088, after)

    def test_one_ms_interval_mutation_fails(self):
        with self.assertRaisesRegex(AssertionError, 'repeat interval'):
            oracle['verify'](trace(51), 1088, 1086)

    def test_release_racing_admission_has_an_exact_count(self):
        admitted = trace().replace('at_ms=500', 'at_ms=476').replace('at_ms=477', 'at_ms=478')
        self.assertEqual(2, oracle['verify'](admitted, 1088, 1086)['expected_deletes'])
        rejected = admitted.replace('tick down_ms=100 at_ms=478', 'skipped down_ms=100 at_ms=478')
        self.assertEqual(1, oracle['verify'](rejected, 1088, 1087)['expected_deletes'])
        with self.assertRaises(AssertionError):
            oracle['verify'](rejected, 1088, 1086)

    def test_missing_release_and_late_tick_fail(self):
        for log in (trace().split('up down_ms')[0], trace().replace('at_ms=500', 'at_ms=450')):
            with self.assertRaises(AssertionError):
                oracle['verify'](log, 1088, 1086)


if __name__ == '__main__':
    unittest.main()
