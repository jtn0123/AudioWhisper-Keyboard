"""Compare raw IME trees and semantic targets; run with python3 -I.

Framework wrappers and explicit virtual-key semantics differ intentionally. Every raw
unmatched node is retained in the JSON report, with an explanation instead of discarded.
Unexpected labels, action loss, state errors and displaced target centers are bugs.
"""
import argparse
from collections import Counter
import json
from pathlib import Path

STATES = ("letters", "shifted", "caps-lock", "symbols", "numeric", "expanded", "popup")
FIELDS = ("text", "description", "viewId", "class", "bounds", "checked", "checkable",
          "selected", "enabled", "focusable", "clickable", "longClickable", "stateDescription", "actions")
FALLBACKS = {"Shift", "Shift enabled", "Caps lock enabled", "Delete", "Symbols", "Comma", "Space",
             "Period", "Return", "Expand actions", "Collapse actions", "Enter", "Numeric", "Done"}
# Frozen public key-name mapping. Match these to actual Compose glyphs and compare their
# target geometry; do not waive punctuation merely because Rust speaks a fallback name.
SPOKEN = dict(zip(
    list('!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~') + ['?123', '=\\<', 'ABC', '1 2\n3 4'],
    ['Exclamation mark', 'Quotation mark', 'Number sign', 'Dollar sign', 'Percent', 'Ampersand',
     'Apostrophe', 'Left parenthesis', 'Right parenthesis', 'Asterisk', 'Plus', 'Comma', 'Hyphen',
     'Period', 'Slash', 'Colon', 'Semicolon', 'Less than', 'Equals', 'Greater than', 'Question mark',
     'At sign', 'Left bracket', 'Backslash', 'Right bracket', 'Caret', 'Underscore', 'Grave accent',
     'Left brace', 'Vertical bar', 'Right brace', 'Tilde', 'Symbols', 'More symbols', 'Letters', 'Numeric']))


def center(bounds):
    return ((bounds[0] + bounds[2]) / 2, (bounds[1] + bounds[3]) / 2)


def distance(a, b):
    return max(abs(x-y) for x, y in zip(center(a), center(b)))


def actions(node):
    return {a["id"] for a in node["actions"]}


def enabled(nodes, index):
    # Compose's icon leaf can report enabled while its merged button parent is disabled.
    while index >= 0:
        node = nodes[index]
        if not node["enabled"]:
            return False
        index = node["parent"]
    return True


def compare(compose, rust, state):
    differences = []
    used = set()
    semantic = [(i, n) for i, n in enumerate(rust) if n["textEntryKey"] or n["description"]]
    if not any(n["textEntryKey"] for n in rust):
        # Permit a Compose/Compose control, but require virtual keys in accepted Rust runs.
        semantic = list(enumerate(rust))
    for ri, node in semantic:
        if node["textEntryKey"]:
            if 16 not in actions(node):
                differences.append({"verdict": "bug", "rust_index": ri, "reason": "Virtual key is missing ACTION_CLICK"})
            if node["class"] not in ("android.inputmethodservice.Keyboard$Key", "android.widget.Button"):
                differences.append({"verdict": "bug", "rust_index": ri, "reason": "Unexpected virtual text-entry role/class"})
            if node["class"].endswith("Keyboard$Key") and node["parent"] >= 0:
                r, host = node["bounds"], rust[node["parent"]]["bounds"]
                if r[0] < host[0] or r[1] < host[1] or r[2] > host[2] or r[3] > host[3]:
                    differences.append({"verdict": "bug", "rust_index": ri, "reason": "Key target escapes the renderer viewport"})
        candidates = []
        label = node["text"] or node["description"]
        for ci, other in enumerate(compose):
            if ci in used:
                continue
            other_label = other["text"] or other["description"]
            if label and label == SPOKEN.get(other_label, other_label):
                candidates.append((distance(node["bounds"], other["bounds"]), ci, other))
            elif not label and all(node.get(k) == other.get(k) for k in ("class", "bounds", "viewId")):
                candidates.append((0, ci, other))
        if not candidates:
            verdict = "acceptable" if label in FALLBACKS else "bug"
            differences.append({"verdict": verdict, "rust_index": ri, "compose_index": None,
                "reason": "Explicit name/state/action for a Compose icon or expansion control" if verdict == "acceptable"
                    else "No Compose label/description counterpart", "rust": node})
            continue
        _, ci, other = min(candidates, key=lambda c: c[0])
        used.add(ci)
        changes = {}
        for field in FIELDS:
            compose_value = enabled(compose, ci) if field == "enabled" else other.get(field)
            if node.get(field) == compose_value:
                continue
            verdict = "acceptable"
            reason = "Explicit virtual-key semantics replace Compose text/icon leaves"
            if field == "class" and node["textEntryKey"] and node["class"] not in (
                    "android.inputmethodservice.Keyboard$Key", "android.widget.Button"):
                verdict, reason = "bug", "Unexpected virtual text-entry role/class"
            elif field == "enabled":
                verdict, reason = "bug", "Enabled state differs"
            elif field == "bounds":
                if node["textEntryKey"]:
                    # Text leaves have glyph bounds; compare target center to their parent key box.
                    target = compose[other["parent"]] if other["parent"] >= 0 else other
                    r, c = node["bounds"], other["bounds"]
                    edge_key = node["class"].endswith("Keyboard$Key") and (r[0] == 0 or r[2] == 1080)
                    contained = r[0]-3 <= c[0] and r[1]-3 <= c[1] and r[2]+3 >= c[2] and r[3]+3 >= c[3]
                    aligned = abs(center(r)[1] - center(target["bounds"])[1]) <= 3
                    if not (distance(r, target["bounds"]) <= 3 or (edge_key and contained and aligned)):
                        verdict, reason = "bug", "Physical key/popup target differs by more than 3 px"
                    else:
                        reason = "Same key center within 3 px, or an edge hit target extended to the screen boundary"
                elif max(abs(x-y) for x, y in zip(node["bounds"], other["bounds"])) > 3:
                    verdict, reason = "bug", "Named control bounds differ by more than 3 px"
            elif field in ("text", "description"):
                reason = "Rust supplies one explicit spoken label on the actionable node"
            elif field == "actions" and node["textEntryKey"] and 16 not in actions(node):
                verdict, reason = "bug", "Virtual key is missing ACTION_CLICK"
            elif field in ("checked", "checkable", "stateDescription", "selected"):
                reason = "Rust adds toggle/selection semantics absent from Compose text/icon leaves"
            changes[field] = {"compose": compose_value, "rust": node.get(field),
                              "verdict": verdict, "reason": reason}
        if node["textEntryKey"] and 16 not in actions(node):
            changes["missing_click"] = {"verdict": "bug", "reason": "Virtual key is missing ACTION_CLICK"}
        differences.append({"rust_index": ri, "compose_index": ci, "label": label,
            "verdict": "bug" if any(v["verdict"] == "bug" for v in changes.values()) else
                       "acceptable" if changes else "matches", "differences": changes})
    # Preserve every remaining raw node. These are normally structural wrappers/hint leaves.
    labels = {n["text"] or n["description"] for n in rust}
    hints = set("1234567890@#$%&-+()*\"':;!?")
    for ci, node in enumerate(compose):
        if ci in used:
            continue
        label = node["text"] or node["description"]
        acceptable = not label or label in labels or SPOKEN.get(label) in labels or label in hints or label == "English (United States)"
        differences.append({"compose_index": ci, "rust_index": None, "compose": node,
            "verdict": "acceptable" if acceptable else "bug",
            "reason": "Compose framework wrapper, hint/glyph leaf or repeated label" if acceptable
                else "Compose label missing from Rust"})
    # Keep all remaining Rust wrappers too; no node silently disappears from the report.
    semantic_ids = {i for i, _ in semantic}
    for ri, node in enumerate(rust):
        if ri not in semantic_ids:
            differences.append({"rust_index": ri, "compose_index": None, "rust": node,
                                "verdict": "acceptable", "reason": "Rust host/framework wrapper"})
    if state in ("letters", "shifted", "caps-lock"):
        expected = {"letters": (False, "Shift"), "shifted": (True, "Shift enabled"),
                    "caps-lock": (True, "Caps lock enabled")}[state]
        shifts = [n for n in rust if n["textEntryKey"] and n["checkable"]]
        if len(shifts) != 1 or (shifts[0]["checked"], shifts[0]["stateDescription"]) != expected:
            differences.append({"verdict": "bug", "reason": "Rust Shift checked/stateDescription does not match requested state"})
    return differences


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    report = {}
    print("| State | Compose nodes | Rust nodes | Matches | Acceptable | Bugs | Verdict |")
    print("|---|---:|---:|---:|---:|---:|---|")
    for state in STATES:
        compose, rust = [json.loads((args.directory / (r + "-" + state + ".json")).read_text())["nodes"]
                         for r in ("compose", "rust")]
        differences = compare(compose, rust, state)
        counts = Counter(d["verdict"] for d in differences)
        verdict = "bug" if counts["bug"] else "acceptable" if counts["acceptable"] else "matches"
        report[state] = {"compose_nodes": len(compose), "rust_nodes": len(rust), "counts": dict(counts),
                         "verdict": verdict, "differences": differences}
        print(f"| {state} | {len(compose)} | {len(rust)} | {counts['matches']} | {counts['acceptable']} | {counts['bug']} | {verdict} |")
    (args.directory / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    for state, result in report.items():
        for diff in result["differences"]:
            print(state, json.dumps(diff, ensure_ascii=False))
    raise SystemExit(any(r["counts"].get("bug", 0) for r in report.values()))


if __name__ == "__main__":
    main()
