"""Adversarial checks for panel fixture targeting and semantic-container normalization."""
from pathlib import Path
import runpy
import unittest

ROOT = Path(__file__).parent
panels = runpy.run_path(str(ROOT / 'panels.py'))
compare = runpy.run_path(str(ROOT / 'compare-panels.py'))


def node(parent, bounds, text=None, kind='android.view.View', enabled=True, visible=True, clickable=False):
    return {'parent': parent, 'bounds': bounds, 'text': text, 'description': None,
            'class': kind, 'enabled': enabled, 'visible': visible, 'clickable': clickable,
            'actions': [{'id': 16}] if clickable else []}


class PanelHarnessTests(unittest.TestCase):
    def test_disabled_native_button_never_targets_the_clickable_root(self):
        nodes = [node(-1, [0, 1562, 1080, 2274], kind='android.widget.FrameLayout', clickable=True),
                 node(0, [287, 1686, 530, 1881], 'Copy', 'android.widget.Button', enabled=False)]
        self.assertEqual((408, 1783), panels['center'](panels['named'](nodes, 'Copy')))

    def test_compose_taps_use_the_visible_label_and_comparison_uses_container_state(self):
        nodes = [node(-1, [0, 1562, 1080, 2274], clickable=True),
                 node(0, [287, 1686, 530, 1881], enabled=False),
                 node(1, [368, 1780, 449, 1823], 'Copy', 'android.widget.TextView')]
        self.assertEqual((408, 1801), panels['center'](panels['named'](nodes, 'Copy')))
        target = compare['targets']({'nodes': nodes})['Copy']
        self.assertFalse(target['enabled'])
        self.assertEqual(nodes[1]['bounds'], target['bounds'])

    def test_partly_visible_tile_keeps_its_label_when_the_text_leaf_is_clipped(self):
        nodes = [node(-1, [0, 1562, 1080, 2274]),
                 node(0, [287, 2227, 530, 2263], enabled=False),
                 node(1, [368, 2369, 449, 2412], 'Copy', 'android.widget.TextView', visible=False)]
        target = compare['targets']({'nodes': nodes})['Copy']
        self.assertFalse(target['enabled'])
        self.assertEqual(nodes[1]['bounds'], target['bounds'])
        nodes[1]['visible'] = False
        self.assertNotIn('Copy', compare['targets']({'nodes': nodes}))

    def test_native_text_node_does_not_expand_to_the_root_bounds(self):
        nodes = [node(-1, [0, 132, 1080, 2274], kind='android.widget.FrameLayout', clickable=True),
                 node(0, [44, 708, 1036, 757], 'Sticky action (1)', 'android.widget.TextView')]
        nodes[1]['description'] = nodes[1]['text']
        self.assertEqual(nodes[1]['bounds'], compare['targets']({'nodes': nodes})['Sticky action (1)']['bounds'])


if __name__ == '__main__':
    suite = unittest.TestSuite([unittest.defaultTestLoader.loadTestsFromTestCase(cls)
                               for cls in (PanelHarnessTests, compare['ComparatorTests'])])
    raise SystemExit(not unittest.TextTestRunner().run(suite).wasSuccessful())
