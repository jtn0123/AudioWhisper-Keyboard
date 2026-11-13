"""Adversarial checks: the comparator must reject an incorrect keyboard."""
from copy import deepcopy
from pathlib import Path
import runpy
import unittest

compare = runpy.run_path(str(Path(__file__).with_name("compare-a11y.py")))["compare"]


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


if __name__ == "__main__":
    unittest.main()
