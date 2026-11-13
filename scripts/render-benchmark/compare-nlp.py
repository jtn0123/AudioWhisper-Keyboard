"""Candidate-strip accessibility comparison with hashed labels only. python3 -I."""
import json
import hashlib
from pathlib import Path
import sys


def compare(compose, rust, state):
    bugs = []
    if [v['labelSha256'] for v in compose] != [v['labelSha256'] for v in rust]:
        bugs.append('Candidate order/labels differ')
    # These are fixed public action buttons, present when an empty strip expands its actions.
    controls = {hashlib.sha256(v.encode()).hexdigest() for v in ('More actions', 'Undo', 'Redo', 'Settings')}
    if state == 'password' and any(v['labelSha256'] not in controls for v in rust):
        bugs.append('Password exposes candidates')
    for i, value in enumerate(rust):
        if not value['enabled'] or not value['clickable'] or not value['clickAction']:
            bugs.append(f'Candidate {i} has no enabled click action')
        x1, y1, x2, y2 = value['bounds']
        if x2 <= x1 or y2 <= y1 or y1 < 1500 or y2 > 1700:
            bugs.append(f'Candidate {i} target is empty or outside strip')
        if i < len(compose) and 'labelBounds' in compose[i]:
            a = compose[i]['labelBounds']
            if abs((x1 + x2) - (a[0] + a[2])) > 6:
                bugs.append(f'Candidate {i} center moved over 3px')
    return [{'verdict': 'bug', 'reason': reason} for reason in bugs]


def main():
    root = Path(sys.argv[1]); results = {}
    for state in ('typing', 'after-autocorrect', 'password'):
        compose = json.loads((root / 'compose' / (state + '-strip.json')).read_text())
        rust = json.loads((root / 'rust' / (state + '-strip.json')).read_text())
        results[state] = compare(compose, rust, state)
    (root / 'accessibility.json').write_text(json.dumps(results, indent=2) + '\n')
    count = sum(map(len, results.values()))
    print('Candidate strip accessibility bug verdicts:', count)
    if count: raise SystemExit(1)


if __name__ == '__main__': main()
