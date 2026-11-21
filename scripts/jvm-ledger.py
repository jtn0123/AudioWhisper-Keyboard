"""Ratchet for the hand-written production JVM files: Kotlin and Java under a module's src/main.

docs/jvm-ledger.tsv has one row per production file: its class, its budget and the reason in its
note. A budget is always the file's current counted lines, so any growth is a visible diff.
Raising a budget is a hand edit of that file with the reason in the note column.

  --check     (default) fail on a production file with no row, a row with no file, a malformed
              row, a file over its budget, or a file under its budget (run --tighten).
  --tighten   lower budgets above the current counts and drop rows for deleted files. It never
              raises a budget and never adds a row. Then it runs the check.
  --summary   files and counted lines per class, and the files over 500 counted lines. The
              500-line rule covers new and changed files, so existing ones only ratchet.

Counted lines are the non-blank lines that still hold code once `//` comments and `/* */` block
comments (nested, and KDoc) are removed. Text inside string and character literals is code, so
`//` or `/*` inside a literal is not a comment. That is the rule of the 500-line file limit.

Run as: python3 -I scripts/jvm-ledger.py [--check | --tighten | --summary]
"""
import argparse
from collections import namedtuple
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LIMIT = 500
HEADER = ("path", "class", "budget", "note")
CLASSES = {
    "A": "Android adapter, stays",
    "M": "mixed adapter, logic to move",
    "P": "portable, moves to Rust",
    "D": "dead, delete",
    "K": "Compose keyboard, deleted at compose-removal step K",
    "S": "Compose settings, deleted at compose-removal steps 5-12",
}
Row = namedtuple("Row", "path cls budget note")

CODE, STR, RAW, BLOCK, LINE = "code", "str", "raw", "block", "line"


def _is_code(ch):
    return not (ch.isspace() or ch == "\ufeff")


def _char_literal_end(line, i):
    """Index just past the character literal whose opening quote is at line[i], or None."""
    j = i + 3 if line[i + 1:i + 2] == "\\" else i + 2
    while j < len(line) and j <= i + 12 and line[j] != "'":
        j += 1
    return j + 1 if j < len(line) and line[j] == "'" else None


def _line_has_code(line, stack):
    """Whether this line holds code. Updates `stack`, whose top is [kind, n]: n is the brace
    count of a ${...} template in code, or the comment depth in a block comment."""
    has = False
    i = 0
    while i < len(line):
        kind, depth = stack[-1]
        if kind == LINE:
            break
        if kind == BLOCK:
            if line.startswith("/*", i):
                stack[-1][1] += 1
                i += 2
            elif line.startswith("*/", i):
                stack[-1][1] -= 1
                i += 2
                if stack[-1][1] == 0:
                    stack.pop()
            else:
                i += 1
            continue
        ch = line[i]
        if kind in (STR, RAW) and line.startswith("${", i):
            stack.append([CODE, 0])
            has = True
            i += 2
        elif kind == STR and ch == "\\":
            has = True
            i += 2
        elif kind == STR and ch == '"':
            stack.pop()
            has = True
            i += 1
        elif kind == RAW and ch == '"':
            run = len(line[i:]) - len(line[i:].lstrip('"'))
            if run >= 3:
                stack.pop()
            has = True
            i += run
        elif kind in (STR, RAW):
            has = has or _is_code(ch)
            i += 1
        elif line.startswith("//", i):
            stack.append([LINE, None])
            break
        elif line.startswith("/*", i):
            stack.append([BLOCK, 1])
            i += 2
        elif ch == '"':
            has = True
            if line.startswith('"""', i):
                stack.append([RAW, None])
                i += 3
            else:
                stack.append([STR, None])
                i += 1
        elif ch == "'" and (end := _char_literal_end(line, i)) is not None:
            has = True
            i = end
        else:
            if depth is not None and ch == "{":
                stack[-1][1] += 1
            elif depth is not None and ch == "}":
                if depth == 0:
                    stack.pop()
                else:
                    stack[-1][1] -= 1
            has = has or _is_code(ch)
            i += 1
    return has


def code_flags(text):
    """One bool per line of `text`: whether that line holds code once comments are removed."""
    stack = [[CODE, None]]
    flags = []
    for line in text.split("\n"):
        flags.append(_line_has_code(line, stack))
        if stack[-1][0] == LINE:
            stack.pop()
        if stack[-1][0] == STR:
            stack.pop()
    return flags


def count_code_lines(text):
    """The counted lines of a Kotlin or Java source text."""
    return sum(code_flags(text))


def is_production(path):
    """Whether a Kotlin or Java path is production: under a module's src/main, not in benchmark."""
    module, sep, _ = path.partition("/src/main/")
    return bool(sep) and "benchmark" not in module.split("/") and path.endswith((".kt", ".java"))


def production_paths(root=ROOT):
    """The tracked production Kotlin and Java files of the repository at root, sorted."""
    out = subprocess.run(["git", "-C", str(root), "ls-files", "-z", "--", "*.kt", "*.java"],
                         check=True, capture_output=True).stdout.decode("utf-8")
    return sorted(path for path in out.split("\0") if is_production(path))


def count_files(paths, root=ROOT):
    """Counted lines per path, and the tracked paths that are missing from the working tree."""
    counts, missing = {}, []
    for path in paths:
        try:
            text = (Path(root) / path).read_text(encoding="utf-8", errors="replace")
        except FileNotFoundError:
            missing.append(path)
            continue
        counts[path] = count_code_lines(text)
    return counts, missing


def parse_ledger(text):
    """The rows of a ledger text, and one message for each malformed line."""
    lines = text.splitlines()
    rows, errors = [], []
    if not lines or tuple(lines[0].split("\t")) != HEADER:
        errors.append("docs/jvm-ledger.tsv: the first line must be the header "
                      "path, class, budget, note (tab-separated)")
    for number, line in enumerate(lines[1:], start=2):
        fields = line.split("\t", 3)
        if len(fields) != 4 or not fields[0] or not fields[3].strip():
            errors.append(f"line {number}: malformed row, expected path, class, budget and note: {line!r}")
        elif fields[1] not in CLASSES:
            errors.append(f"line {number}: {fields[0]} has unknown class {fields[1]!r}")
        elif not re.fullmatch(r"0|[1-9][0-9]*", fields[2]):
            errors.append(f"line {number}: {fields[0]} has a malformed budget {fields[2]!r}")
        else:
            rows.append(Row(fields[0], fields[1], int(fields[2]), fields[3]))
    return rows, errors


def format_ledger(rows):
    """The ledger text for rows, sorted by path."""
    lines = ["\t".join(HEADER)]
    lines += [f"{row.path}\t{row.cls}\t{row.budget}\t{row.note}" for row in sorted(rows, key=lambda r: r.path)]
    return "\n".join(lines) + "\n"


def find_problems(rows, errors, counts, missing=()):
    """Every way the ledger does not match the production files. Empty when it matches."""
    problems = list(errors)
    problems += [f"tracked but missing from the working tree: {path}" for path in missing]
    by_path = {}
    for row in rows:
        if row.path in by_path:
            problems.append(f"duplicate row: {row.path}")
        by_path[row.path] = row
    for path in sorted(counts):
        n = counts[path]
        row = by_path.get(path)
        if row is None:
            problems.append(f"no row: {path} has {n} counted lines; add a row by hand with its "
                            "class and the reason in the note column")
        elif n > row.budget:
            problems.append(f"grew: {path} has {n} counted lines, budget {row.budget}. Raising a "
                            "budget is a hand edit of docs/jvm-ledger.tsv with the reason in the note column")
        elif n < row.budget:
            problems.append(f"shrank: {path} has {n} counted lines, budget {row.budget}. "
                            "Run python3 -I scripts/jvm-ledger.py --tighten")
    for path in sorted(by_path):
        if path not in counts:
            problems.append(f"row for a file that is not a production file (deleted, renamed or "
                            f"moved out of src/main): {path}")
    return problems


def tighten(rows, counts):
    """The rows with over-generous budgets lowered to the current counts, without rows for deleted
    files, and the changes made. A budget below its count is kept as it is: never raised here."""
    kept, changes = [], []
    for row in rows:
        if row.path not in counts:
            changes.append(f"removed row for a file that is gone: {row.path}")
            continue
        n = counts[row.path]
        if row.budget > n:
            changes.append(f"budget {row.budget} -> {n}: {row.path}")
            row = row._replace(budget=n)
        kept.append(row)
    return kept, changes


def summary_lines(counts, rows):
    """A table of files and counted lines per class, with a total."""
    cls_of = {row.path: row.cls for row in rows}
    totals = {cls: [0, 0] for cls in CLASSES}
    unlisted = [0, 0]
    for path, n in counts.items():
        bucket = totals.get(cls_of.get(path), unlisted)
        bucket[0] += 1
        bucket[1] += n
    lines = [f"{'class':<6}{'files':>7}{'counted lines':>15}  meaning"]
    for cls, meaning in CLASSES.items():
        lines.append(f"{cls:<6}{totals[cls][0]:>7}{totals[cls][1]:>15}  {meaning}")
    files = sum(bucket[0] for bucket in totals.values()) + unlisted[0]
    lines_total = sum(bucket[1] for bucket in totals.values()) + unlisted[1]
    lines.append(f"{'total':<6}{files:>7}{lines_total:>15}")
    if unlisted[0]:
        lines.append(f"unlisted: {unlisted[0]} file(s), {unlisted[1]} counted lines; --check reports them")
    over = sorted((n, path) for path, n in counts.items() if n > LIMIT)
    lines.append(f"over {LIMIT} counted lines: {len(over)} file(s)")
    lines += [f"  {n:>5}  {path}" for n, path in sorted(over, reverse=True)]
    return lines


def main(argv=None, root=ROOT):
    parser = argparse.ArgumentParser(description="Check, tighten or summarize docs/jvm-ledger.tsv.")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="fail on any mismatch (default)")
    mode.add_argument("--tighten", action="store_true", help="lower budgets to the current counts, drop rows of deleted files, then check")
    mode.add_argument("--summary", action="store_true", help="print files and counted lines per class")
    args = parser.parse_args(argv)
    ledger = Path(root) / "docs" / "jvm-ledger.tsv"
    if ledger.is_file():
        rows, errors = parse_ledger(ledger.read_text(encoding="utf-8"))
    else:
        rows, errors = [], ["docs/jvm-ledger.tsv is missing"]
    counts, missing = count_files(production_paths(root), root)
    if args.summary:
        print("\n".join(summary_lines(counts, rows)))
        return 0
    if args.tighten:
        if errors or missing:
            print("\n".join(errors + [f"tracked but missing from the working tree: {p}" for p in missing]),
                  file=sys.stderr)
            return 1
        rows, changes = tighten(rows, counts)
        for change in changes:
            print(change)
        if changes:
            ledger.write_text(format_ledger(rows), encoding="utf-8")
        print(f"tightened: {len(changes)} change(s)")
    problems = find_problems(rows, errors, counts, missing)
    if problems:
        print(f"docs/jvm-ledger.tsv does not match the production JVM files ({len(problems)} problem(s)):",
              file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        return 1
    print(f"docs/jvm-ledger.tsv matches {len(counts)} production files and {len(rows)} rows")
    return 0


if __name__ == "__main__":
    sys.exit(main())
