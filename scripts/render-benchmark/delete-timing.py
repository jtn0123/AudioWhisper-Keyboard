"""Exact delivered-delete oracle from timestamp-only debug scheduling observations.

The coroutine delays AFTER each actual repeat, so missed wall-clock slots are not repeats.
A tick is an actual scheduler wake delivered to the repeat callback. Independently validate
all scheduled deadlines against the unscaled Android interval (NO_ACTION delete fixture),
then count native-admitted ticks whose attempts precede completed release. No tolerance or fixed count.
"""
import re

PATTERN = re.compile(r'(down|hold|attempt|tick|skipped|repeat|up) down_ms=(\d+) at_ms=(\d+)(?: due_ms=(\d+) reference_due_ms=(\d+))?')


def expected(log):
    events = [(phase, int(down), int(at), int(due) if due else None, int(reference) if reference else None)
              for phase, down, at, due, reference in PATTERN.findall(log)]
    downs = [e for e in events if e[0] == 'down']
    assert len(downs) == 1, 'Expected exactly one delete hold'
    start = downs[0][1]
    assert all(e[1] == start for e in events), 'Mixed delete holds'
    ups = [e for e in events if e[0] == 'up']
    holds = [e for e in events if e[0] == 'hold']
    assert len(ups) == len(holds) == 1, 'Missing down/up/hold observations'
    up = ups[0][2]
    hold = holds[0]
    assert hold[3] == hold[4] == hold[2] + 300, 'Long-press delay changed'
    attempts = [e for e in events if e[0] == 'attempt']
    outcomes = [e for e in events if e[0] in ('tick', 'skipped')]
    ticks = [e for e in outcomes if e[0] == 'tick']
    waits = [e for e in events if e[0] == 'repeat']
    assert len(waits) == len(attempts) == len(outcomes), 'Unpaired repeat scheduling observation'
    due = hold[3]
    for attempt, outcome, wait in zip(attempts, outcomes, waits):
        assert due <= attempt[2], 'Attempt before deadline'
        assert attempt[2] <= outcome[2] <= wait[2], 'Repeat timestamps out of order'
        if outcome[0] == 'tick':
            # UP can win before REPEAT_RESULT, or arrive between admission and logging/
            # delivery. Count exactly the admitted ticks, including an already-admitted
            # tick whose log lands after UP. An attempt starting after completed UP cannot
            # be admitted. This observes timer admission independently of editor effects.
            assert attempt[2] <= up, 'Tick attempted after completed release'
        assert wait[3] == wait[4], 'Production repeat interval differs from Android schedule'
        due = wait[3]
    assert ticks and start <= hold[2] < up, 'No actual repeat opportunities'
    return {'down_ms': start, 'up_ms': up, 'expected_deletes': len(ticks),
            'ticks_ms': [e[2] for e in ticks], 'attempts_ms': [e[2] for e in attempts],
            'skipped_ms': [e[2] for e in outcomes if e[0] == 'skipped'], 'waits_ms': [e[2] for e in waits],
            'deadlines_ms': [hold[3]] + [e[3] for e in waits]}


def verify(log, before, after):
    result = expected(log)
    assert before - after == result['expected_deletes'], 'Delivered deletes differ from exact scheduled opportunities'
    return result
