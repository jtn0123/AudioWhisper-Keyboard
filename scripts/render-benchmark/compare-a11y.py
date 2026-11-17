"""Compare raw IME trees and semantic targets; run with python3 -I.

Framework wrappers and explicit virtual-key semantics differ intentionally. Every raw
unmatched node is retained in the JSON report, with an explanation instead of discarded.
Unexpected labels, action loss, state errors and displaced target centers are bugs.

The Rust tree must also read in screen order (`order`): the strip above the keys first, then the
keys row by row, each row left to right, with a popup's choices straight after the key that
opened them, and each node's traversal hint naming the node before it. `--order DIR` checks
label-free captures from a11y-order.py and prints only roles, indices and bounds.
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


KEY = "android.inputmethodservice.Keyboard$Key"


def reads_before(a, b):
    """Whether b may be read straight after a: later in the same row, or in a later row. A node
    shares a row with another while it starts above that node's middle, Rust's own rule."""
    if b[1] < (a[1] + a[3]) / 2 and a[1] < (b[1] + b[3]) / 2:
        return b[0] >= a[0]
    return b[1] > a[1]


def role(node):
    if node["class"] == KEY:
        return "key"
    if node["textEntryKey"]:
        return "popup item"
    return node["class"].rsplit(".", 1)[-1].lower()


def opener(nodes, keys, popup):
    """Keys that could have opened a popup: under its span, in the row nearest its bottom edge."""
    block = [nodes[i]["bounds"] for i in popup]
    left, right, bottom = min(b[0] for b in block), max(b[2] for b in block), max(b[3] for b in block)
    under = [k for k in keys if left <= center(nodes[k]["bounds"])[0] <= right]
    if not under:
        return []
    gap = min(abs(nodes[k]["bounds"][1] - bottom) for k in under)
    return [k for k in under if abs(nodes[k]["bounds"][1] - bottom) == gap]


def order(nodes):
    """Reading-order bugs in a Rust IME tree, by dump index. Labels are never read."""
    hosts = {n["parent"] for n in nodes if n["class"] == KEY}
    if len(hosts) != 1:
        return [] if not hosts else [{"verdict": "bug", "reason": "Reading order: keys under more than one host"}]
    host = hosts.pop()
    children = [i for i, n in enumerate(nodes) if n["parent"] == host]
    keys = [i for i in children if nodes[i]["class"] == KEY]
    popup = [i for i in children if nodes[i]["textEntryKey"] and nodes[i]["class"] != KEY]
    top = min(nodes[i]["bounds"][1] for i in keys)
    strip = [i for i in children if not nodes[i]["textEntryKey"] and nodes[i]["bounds"][3] <= top]
    issues = []
    def bug(reason, *indices):
        issues.append({"verdict": "bug", "reason": "Reading order: " + reason, "rust_indices": list(indices)})
    late = [i for i in strip if children.index(i) > children.index(keys[0])]
    if late:
        bug("the strip above the keys is read after a key", *late)
    for group, name in ((strip, "strip nodes"), (keys, "keys")):
        for a, b in zip(group, group[1:]):
            if not reads_before(nodes[a]["bounds"], nodes[b]["bounds"]):
                bug(name + " are not read row by row, left to right", a, b)
    if popup:
        at = [children.index(i) for i in popup]
        if at != list(range(at[0], at[0] + len(at))):
            bug("a popup's choices are not read together", *popup)
        if at[0] == 0 or children[at[0] - 1] not in opener(nodes, keys, popup):
            bug("a popup's choices do not follow the key that opened them", *popup)
        for a, b in zip(popup, popup[1:]):
            if not reads_before(nodes[a]["bounds"], nodes[b]["bounds"]):
                bug("a popup's choices are not read row by row, left to right", a, b)
    # Without hints Android follows the child order; when Rust gives them, they must agree.
    if any(nodes[i].get("traversalAfter", -1) != -1 for i in children):
        for at, i in enumerate(children):
            if nodes[i].get("traversalAfter") != (children[at - 1] if at else -1):
                bug("the traversal hint does not name the node read before", i)
    return issues


def order_report(directory):
    """Print each label-free capture's reading order and its bugs; return whether any were found."""
    found = False
    for path in sorted(directory.glob("*.json")):
        data = json.loads(path.read_text())
        if "nodes" not in data:
            continue
        nodes = data["nodes"]
        keys = [n["parent"] for n in nodes if n["class"] == KEY]
        host = keys[0] if keys else None
        print(f"## {path.stem}")
        print("| Read | Index | Role | Bounds | Traversal after |")
        print("|---:|---:|---|---|---:|")
        for read, i in enumerate(i for i, n in enumerate(nodes) if n["parent"] == host):
            n = nodes[i]
            print(f"| {read} | {i} | {role(n)} | {n['bounds']} | {n.get('traversalAfter', '')} |")
        for issue in order(nodes):
            found = True
            print(path.stem, json.dumps(issue))
    return found


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
                    host_bounds = rust[node["parent"]]["bounds"] if node["parent"] >= 0 else [0, 0, 1080, 2400]
                    edge_key = node["class"].endswith("Keyboard$Key") and (r[0] == host_bounds[0] or r[2] == host_bounds[2])
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
    differences.extend(order(rust))
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
    parser.add_argument("--order", action="store_true", help="check label-free a11y-order.py captures")
    args = parser.parse_args()
    if args.order:
        raise SystemExit(order_report(args.directory))
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
