"""Adversarial checks: the comparator must reject an incorrect keyboard."""
from copy import deepcopy
from pathlib import Path
import runpy
import unittest

module = runpy.run_path(str(Path(__file__).with_name("compare-a11y.py")))
compare, order = module["compare"], module["order"]


def fixture():
    node = {"parent": -1, "text": "e", "description": None, "viewId": None,
            "class": "android.widget.TextView", "bounds": [0, 0, 100, 100],
            "checked": False, "checkable": False, "selected": False, "enabled": True,
            "focusable": False, "clickable": False, "longClickable": False,
            "stateDescription": None, "actions": [{"id": 64, "label": None}], "textEntryKey": False}
    rust = deepcopy(node)
    rust.update({"class": "android.inputmethodservice.Keyboard$Key", "textEntryKey": True,
                 "description": "e", "clickable": True, "focusable": True,
                 "actions": [{"id": 64, "label": None}, {"id": 16, "label": None}]})
    return [node], [rust]


class ComparatorTest(unittest.TestCase):
    def rejects(self, mutation):
        compose, rust = fixture()
        mutation(rust)
        self.assertTrue(any(d["verdict"] == "bug" for d in compare(compose, rust, "popup")))

    def test_explicit_semantics_are_acceptable(self):
        self.assertFalse(any(d["verdict"] == "bug" for d in compare(*fixture(), "popup")))

    def test_wrong_label_is_a_bug(self):
        self.rejects(lambda ns: ns[0].update(text="wrong", description="wrong"))

    def test_missing_key_is_a_bug(self):
        self.rejects(lambda ns: ns.clear())

    def test_displaced_target_is_a_bug(self):
        self.rejects(lambda ns: ns[0].update(bounds=[5, 0, 105, 100]))

    def test_missing_click_is_a_bug(self):
        self.rejects(lambda ns: ns[0].update(actions=[{"id": 64, "label": None}]))

    def test_wrong_role_is_a_bug(self):
        self.rejects(lambda ns: ns[0].update({"class": "android.widget.EditText"}))

    def test_disabled_key_is_a_bug(self):
        self.rejects(lambda ns: ns[0].update(enabled=False))

    def test_key_in_system_navigation_is_a_bug(self):
        compose, rust = fixture()
        host = deepcopy(rust[0])
        host.update(text=None, description=None, textEntryKey=False, **{"class": "android.widget.FrameLayout"})
        rust[0].update(parent=0, bounds=[0, 0, 100, 120])
        rust.insert(0, host)
        self.assertTrue(any(d["reason"] == "Key target escapes the renderer viewport"
                            for d in compare(compose, rust, "popup") if "reason" in d))

    def test_wrong_shift_state_is_a_bug(self):
        compose, rust = fixture()
        rust[0].update(text="Shift", description="Shift", checkable=True, checked=False,
                       stateDescription="Shift")
        self.assertTrue(any(d["verdict"] == "bug" for d in compare(compose, rust, "caps-lock")))

    def test_spoken_symbol_must_match_its_real_glyph_and_geometry(self):
        compose, rust = fixture()
        compose[0].update(text="@")
        rust[0].update(text="At sign", description="At sign")
        self.assertFalse(any(d["verdict"] == "bug" for d in compare(compose, rust, "symbols")))
        rust[0].update(bounds=[5, 0, 105, 100])
        self.assertTrue(any(d["verdict"] == "bug" for d in compare(compose, rust, "symbols")))

    def test_missing_spoken_symbol_is_a_bug(self):
        compose, rust = fixture()
        compose[0].update(text="@")
        rust[0].update(text="Hyphen", description="Hyphen")
        self.assertTrue(any(d["verdict"] == "bug" for d in compare(compose, rust, "symbols")))

    def test_disabled_compose_icon_inherits_its_button_state(self):
        compose, rust = fixture()
        compose[0].update(text="Voice input", description="Voice input", parent=0)
        parent = deepcopy(compose[0])
        parent.update(parent=-1, text=None, description=None, enabled=False)
        compose.insert(0, parent)
        rust[0].update(text="Voice input", description="Voice input", textEntryKey=False,
                       enabled=False, actions=[{"id": 64, "label": None}], **{"class": "android.widget.Button"})
        self.assertFalse(any(d["verdict"] == "bug" for d in compare(compose, rust, "numeric")))
        rust[0].update(enabled=True)
        self.assertTrue(any(d["verdict"] == "bug" for d in compare(compose, rust, "numeric")))

    def test_compact_and_landscape_edges_follow_the_renderer_viewport(self):
        for left, right in ((216,1080),(132,2400)):
            compose,rust=fixture()
            box=deepcopy(compose[0]);box.update(text=None,bounds=[left+44,0,left+130,100])
            compose[0].update(parent=0,bounds=[left+74,30,left+99,70]);compose.insert(0,box)
            host=deepcopy(rust[0]);host.update(text=None,description=None,textEntryKey=False,bounds=[left,0,right,100],**{"class":"android.widget.FrameLayout"})
            rust[0].update(parent=0,bounds=[left,0,left+130,100]);rust.insert(0,host)
            self.assertFalse(any(row['verdict']=='bug' for row in compare(compose,rust,'onehand')))
            rust[1]['bounds'][0]+=1
            self.assertTrue(any(row['verdict']=='bug' for row in compare(compose,rust,'onehand')))


def screen(after=False):
    """A host, then a strip of mic, word and arrow, then two rows of two keys, in screen order."""
    def node(parent, cls, bounds, key=False):
        return {"parent": parent, "class": cls, "bounds": bounds, "textEntryKey": key}
    keyboard = "android.inputmethodservice.Keyboard$Key"
    nodes = [node(-1, "android.widget.FrameLayout", [0, 0, 300, 140]),
             node(0, "android.widget.Button", [0, 0, 40, 40]),
             node(0, "android.widget.Button", [40, 0, 270, 40]),
             node(0, "android.widget.Button", [270, 0, 300, 40]),
             node(0, keyboard, [0, 40, 150, 90], True), node(0, keyboard, [150, 40, 300, 90], True),
             node(0, keyboard, [0, 90, 150, 140], True), node(0, keyboard, [150, 90, 300, 140], True)]
    for i, n in enumerate(nodes):
        n["traversalAfter"] = (i - 1 if i > 1 else -1) if after else -1
    return nodes


class ReadingOrderTest(unittest.TestCase):
    def test_screen_order_passes_with_and_without_hints(self):
        self.assertEqual(order(screen()), [])
        self.assertEqual(order(screen(after=True)), [])

    def test_keys_read_before_the_strip_are_a_bug(self):
        nodes = screen()
        nodes[1:] = nodes[4:] + nodes[1:4]
        self.assertTrue(any("after a key" in d["reason"] for d in order(nodes)))

    def test_the_arrow_before_the_words_to_its_left_is_a_bug(self):
        nodes = screen()
        nodes[2], nodes[3] = nodes[3], nodes[2]
        self.assertTrue(any("strip nodes" in d["reason"] for d in order(nodes)))

    def test_a_lower_row_before_a_higher_one_is_a_bug(self):
        nodes = screen()
        nodes[4:6], nodes[6:8] = nodes[6:8], nodes[4:6]
        self.assertTrue(any("keys are" in d["reason"] for d in order(nodes)))

    def test_popup_choices_follow_their_key_as_one_block(self):
        nodes = screen()
        choices = [{"parent": 0, "class": "android.widget.Button", "bounds": [x, 0, x + 40, 40],
                    "textEntryKey": True} for x in (150, 190)]
        self.assertEqual(order(nodes[:6] + choices + nodes[6:]), [])
        self.assertTrue(order(nodes + choices))
        self.assertTrue(order(nodes[:6] + choices[:1] + nodes[6:7] + choices[1:] + nodes[7:]))

    def test_a_hint_naming_another_node_is_a_bug(self):
        nodes = screen(after=True)
        nodes[5]["traversalAfter"] = 2
        self.assertTrue(any("traversal hint" in d["reason"] for d in order(nodes)))
        nodes[5]["traversalAfter"] = -2
        self.assertTrue(order(nodes))


if __name__ == "__main__":
    unittest.main()
