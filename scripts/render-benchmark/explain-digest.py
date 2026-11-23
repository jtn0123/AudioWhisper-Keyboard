"""Explain a wrong 300-press digest (E29) from the digest alone, without reading the editor.

run.sh presses q w e r a s d f z x c v 25 times each, so the expected text is known. This
hashes simple variants of it until one matches each digest file given: one key typed as its
hint symbol (a tap read as a long press), two keys typed as their hints, one key typed as any other
character, two adjacent keys swapped, or one key dropped and another typed. Prints, per file, the
units and the characters involved, which are only the benchmark's fixed synthetic keys. Run with
python3 -I.
"""
import argparse
import hashlib
import json
from pathlib import Path
import string

KEYS = "qwerasdfzxcv"
# Each key's hint on the English letters layout: the first character its long press gives.
HINTS = dict(zip(KEYS, "1234@#$%*\"':"))
EXPECTED = "".join(key * 25 for key in KEYS)
OTHERS = string.ascii_letters + string.digits + string.punctuation + " "


def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()


def variants():
    """(text, explanation) for every variant, the likeliest kind first."""
    for at, key in enumerate(EXPECTED):
        yield EXPECTED[:at] + HINTS[key] + EXPECTED[at + 1:], f"unit {at}: {key} typed as its hint {HINTS[key]}"
    for first in range(len(EXPECTED)):
        for second in range(first + 1, len(EXPECTED)):
            text = list(EXPECTED)
            text[first], text[second] = HINTS[text[first]], HINTS[text[second]]
            yield "".join(text), (f"units {first} and {second}: {EXPECTED[first]} and {EXPECTED[second]} "
                                  f"typed as their hints {text[first]} and {text[second]}")
    for at, key in enumerate(EXPECTED):
        for other in OTHERS:
            if other not in (key, HINTS[key]):
                yield EXPECTED[:at] + other + EXPECTED[at + 1:], f"unit {at}: {key} typed as {other}"
    for at in range(len(EXPECTED) - 1):
        yield EXPECTED[:at] + EXPECTED[at + 1] + EXPECTED[at] + EXPECTED[at + 2:], f"units {at} and {at + 1} swapped"
    for dropped in range(len(EXPECTED)):
        rest = EXPECTED[:dropped] + EXPECTED[dropped + 1:]
        for at in range(len(EXPECTED)):
            for key in KEYS:
                yield rest[:at] + key + rest[at:], f"unit {dropped} dropped, {key} typed at {at}"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("digests", nargs="+", type=Path, help="rust-typing-digest.json files")
    args = parser.parse_args()
    wanted = {}
    for path in args.digests:
        digest = json.loads(path.read_text())
        if digest["sha256"] == sha(EXPECTED):
            print(f"{path}: the expected text")
        else:
            wanted.setdefault(digest["sha256"], []).append(path)
    for text, why in variants():
        if not wanted:
            break
        for path in wanted.pop(sha(text), []):
            print(f"{path}: {why}")
    for paths in wanted.values():
        for path in paths:
            print(f"{path}: no simple explanation")


if __name__ == "__main__":
    main()
