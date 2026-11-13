"""Compare panel targets, effective enabled states, actions and geometry across renderers.

Raw wrapper/glyph nodes are not equivalent to a virtual button. Compare the closest
interactive container; headers compare content rectangles. Unexplained differences are bugs.
"""
import argparse
import copy
import json
from pathlib import Path
import unittest

EXTRA_NAMES = {'recently used', 'smileys emotion', 'people body', 'animals nature', 'food drink',
               'travel places', 'activities', 'objects', 'symbols', 'flags', 'Backspace', 'Back',
               'Dismiss', 'Actions', 'Emoji', 'Pin', 'Unpin', 'Move left', 'Move right', 'Delete', 'Collapse actions'}


def label(node):
    return node.get('description') or node.get('text')


def target(nodes, index):
    node = nodes[index]
    result = node
    if node.get('class') in ('android.widget.Button', 'android.widget.Switch') or node.get('description') == node.get('text') or any(a['id'] == 16 for a in node.get('actions', [])):
        return node
    parent = node['parent']
    while parent >= 0:
        ancestor = nodes[parent]
        if ancestor.get('class', '').endswith(('RustKeyboardView', 'ComposeInputView')) or ancestor.get('class') == 'android.widget.FrameLayout':
            break
        if ancestor.get('class') in ('android.widget.Button', 'android.widget.Switch') or any(a['id'] == 16 for a in ancestor.get('actions', [])):
            result = ancestor
            break
        # Editor/disabled tiles have no click action, but their immediate semantics
        # wrapper carries the button state and its square or viewport-clipped bounds.
        b = ancestor['bounds']
        if 150 <= b[2] - b[0] <= 260 and 0 < b[3] - b[1] <= 260:
            result = ancestor
            break
        parent = ancestor['parent']
    return result


def targets(data):
    nodes = data['nodes']
    out = {}
    modal = next((i for i, n in enumerate(nodes) if n.get('text') == 'Customize action order' or n.get('description') == 'Customize action order'), None)
    sheet = None
    if modal is not None and nodes[modal].get('description') != nodes[modal].get('text'):
        parent = nodes[modal]['parent']
        while parent >= 0:
            b = nodes[parent]['bounds']
            if b[0] == 0 and b[2] == 1080 and 132 < b[1] < 1562 and b[3] == 2274:
                sheet = parent
                break
            parent = nodes[parent]['parent']
    for index, node in enumerate(nodes):
        name = label(node)
        if sheet is not None:
            parent = index
            while parent >= 0 and parent != sheet:
                parent = nodes[parent]['parent']
            if parent != sheet:
                continue
        if not name:
            continue
        if name == '!! invalid !!':
            out[name] = node
            continue
        item = dict(target(nodes, index))
        if not item['visible'] or item['bounds'][1] >= 2274:
            continue
        # Keep the semantic label even when selecting its unlabeled clickable wrapper.
        item['name'] = name
        out.setdefault(name, item)
    return out


def compare(compose, rust):
    expected, actual = targets(compose), targets(rust)
    results = []
    for name, left in expected.items():
        right = actual.get(name)
        if right is None:
            results.append({'target': name, 'verdict': 'bug', 'reason': 'missing label'})
            continue
        reasons = []
        if left['enabled'] != right['enabled']:
            reasons.append('enabled state differs')
        if any(a['id'] == 16 for a in left.get('actions', [])) and not any(a['id'] == 16 for a in right.get('actions', [])):
            reasons.append('click action missing')
        a, b = left['bounds'], right['bounds']
        if abs((a[0] + a[2] - b[0] - b[2]) / 2) > 3 or abs((a[1] + a[3] - b[1] - b[3]) / 2) > 3:
            reasons.append('target center differs by more than 3px')
        results.append({'target': name, 'verdict': 'bug' if reasons else ('matches' if left == right else 'acceptable'),
                        'reason': '; '.join(reasons) if reasons else 'same named target, state, action and center; wrapper/container semantics may differ'})
    for name in actual.keys() - expected.keys():
        allowed = name in EXTRA_NAMES
        results.append({'target': name, 'verdict': 'acceptable' if allowed else 'bug',
                        'reason': 'explicit native category/control label' if allowed else 'unexplained extra label'})
    return {'bug_count': sum(row['verdict'] == 'bug' for row in results), 'targets': results}


class ComparatorTests(unittest.TestCase):
    def fixture(self):
        node = {'parent': -1, 'class': 'android.widget.Button', 'text': 'Copy', 'description': None,
                'bounds': [100, 1700, 200, 1800], 'enabled': True, 'visible': True, 'actions': [{'id': 16}]}
        return {'nodes': [node]}

    def test_matches(self):
        self.assertEqual(0, compare(self.fixture(), self.fixture())['bug_count'])

    def test_rejects_each_wrong_target_field(self):
        changes = ({'text': 'Wrong'}, {'enabled': False}, {'actions': []}, {'bounds': [300, 1700, 400, 1800]}, {'visible': False})
        for change in changes:
            left, right = self.fixture(), self.fixture()
            right['nodes'][0].update(change)
            self.assertGreater(compare(left, right)['bug_count'], 0)

    def test_rejects_unexplained_extra_label(self):
        left, right = self.fixture(), self.fixture()
        extra = copy.deepcopy(right['nodes'][0]); extra['text'] = '!! invalid !!'; right['nodes'].append(extra)
        self.assertGreater(compare(left, right)['bug_count'], 0)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('root')
    args = parser.parse_args()
    root = Path(args.root)
    results = {}
    for theme in ('day', 'night'):
        for state in ('emoji-category-1', 'people', 'tone-popup', 'overflow', 'editing', 'edit'):
            data = [json.loads((root / (theme + '-' + renderer) / (state + '.json')).read_text()) for renderer in ('compose', 'rust')]
            results[theme + '-' + state] = compare(*data)
    (root / 'comparison.json').write_text(json.dumps(results, indent=2) + '\n')
    counts = {state: value['bug_count'] for state, value in results.items()}
    print(json.dumps(counts))
    if any(counts.values()):
        raise SystemExit(1)


if __name__ == '__main__':
    main()
