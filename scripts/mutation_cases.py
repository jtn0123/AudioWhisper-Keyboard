"""Whether every mutation script's cases still apply, found without building or changing anything.

    python3 -I -B scripts/mutation_cases.py                  # every scripts/*-mutation*.py (the gate's
                                                             # checkMutationCases)
    python3 -I scripts/typing-plan-mutations.py --check      # one script's cases

A case applies when every file it edits exists and each search text occurs exactly as often as the
case expects: once, unless it says otherwise. An edit with no search text makes its file, so it
applies while that file does not exist. A case's edits to one file are found in turn, each in
the text the ones before it left, as the scripts apply them. A case whose line has moved stops a full
run of its script at that case, hours in; this finds every such case in seconds, so the gate can.

A script that knows a case is stale names it in STALE, with the reason and who is fixing it. The
check fails on a STALE case that applies again, and on a STALE name that is no case, so the list can
neither hide a new failure nor outlive the fix.

Each script builds its cases as Edits in `check_edits()`, from the same tables its run reads, and
calls `check_flag()` first in its main block.
"""
import importlib.util
from pathlib import Path
import sys
from typing import Callable, NamedTuple, Optional, Union

ROOT = Path(__file__).resolve().parent.parent
SCRIPTS = ROOT / 'scripts'


class Edit(NamedTuple):
    """One change a case makes: `old` replaced by `new`, where `old` must occur `count` times, or a
    file made with `new` as its text when `old` is empty and the file does not exist. A script whose
    cases are functions of the source gives `change` instead, which raises AssertionError or
    ValueError when its text is not there."""
    case: str
    path: Union[Path, str]
    old: str = ''
    new: str = ''
    count: int = 1
    change: Optional[Callable[[str], str]] = None


def rows(cases):
    """The Edits of cases written as (name, file, old, new, ...) rows."""
    return [Edit(case[0], case[1], case[2], case[3]) for case in cases]


def table(cases):
    """The Edits of cases written as {name: (file, old, new, ...)}."""
    return [Edit(name, case[0], case[1], case[2]) for name, case in cases.items()]


def replace_once(old, new):
    """A change replacing `old`, which must occur exactly once, with `new`."""
    def change(source):
        assert source.count(old) == 1, 'expected exactly one ' + repr(old)
        return source.replace(old, new)
    return change


def _file(path):
    path = Path(path)
    return path if path.is_absolute() else ROOT / path


def _shown(path):
    return str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path)


def _apply(edit, text):
    """The text after `edit`, and why it does not apply, or None when it does."""
    if edit.change is not None:
        try:
            changed = edit.change(text)
        except (AssertionError, ValueError) as error:
            return text, str(error) or 'its change does not apply'
        return changed, None if changed != text else 'its change changes nothing'
    if not edit.old:
        return text, 'its search text is empty'
    found = text.count(edit.old)
    if found != edit.count:
        lines = [line.strip() for line in edit.old.splitlines() if line.strip()] or [edit.old]
        return text, f'{lines[0][:60]!r} found {found} times, not {edit.count}'
    if edit.old == edit.new:
        return text, 'it changes nothing'
    return text.replace(edit.old, edit.new, 1), None


def problems(edits, stale=None):
    """What is wrong with `edits` and `stale`, one line per case; empty when every case applies, and
    every case in `stale` is a case that does not."""
    stale = dict(stale or {})
    cases = {}
    for edit in edits:
        cases.setdefault(edit.case, []).append(edit)
    found = [f'{name}: in STALE, but no case has that name' for name in stale if name not in cases]
    for name, changes in cases.items():
        texts, wrong = {}, []
        for edit in changes:
            path = _file(edit.path)
            if path not in texts:
                texts[path] = path.read_text(encoding='utf-8') if path.is_file() else None
            if texts[path] is None and not edit.old and edit.new and edit.change is None:
                texts[path] = edit.new  # The edit makes the file.
                continue
            if texts[path] is None:
                wrong.append(f'{_shown(path)} does not exist')
                continue
            texts[path], why = _apply(edit, texts[path])
            if why:
                wrong.append(f'{_shown(path)}: {why}')
        if name in stale:
            if not wrong:
                found.append(f'{name}: applies again; take it out of STALE ({stale[name]})')
        elif wrong:
            found.append(f'{name}: ' + '; '.join(wrong))
    return found


def check_script(name, module):
    """Prints what is wrong with one script's cases, or that they all apply; returns its exit status."""
    if not callable(getattr(module, 'check_edits', None)):
        print(f'{name}: has no check_edits(), so its cases cannot be checked')
        return 1
    edits = module.check_edits()
    stale = getattr(module, 'STALE', None) or {}
    found = problems(edits, stale)
    for line in found:
        print(f'{name}: {line}')
    if not found:
        known = f', and the {len(stale)} in STALE still do not' if stale else ''
        print(f'{name}: all {len({edit.case for edit in edits}) - len(stale)} cases apply{known}')
    return 1 if found else 0


def check_flag():
    """Checks the running script's cases and exits with the result when it was given --check;
    returns otherwise."""
    if '--check' in sys.argv[1:]:
        sys.exit(check_script(Path(sys.argv[0]).name, sys.modules['__main__']))


def load(path):
    """A mutation script as a module, without running its main block."""
    spec = importlib.util.spec_from_file_location(path.stem.replace('-', '_'), path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    sys.dont_write_bytecode = True
    # The scripts import this file as mutation_cases: give them this copy, not a second one.
    sys.modules.setdefault('mutation_cases', sys.modules[__name__])
    status = 0
    scripts = sorted(SCRIPTS.glob('*-mutation*.py'))
    for path in scripts:
        try:
            module = load(path)
        except Exception as error:  # A script that cannot load cannot be checked: say which, and why.
            print(f'{path.name}: does not load: {type(error).__name__}: {error}')
            status = 1
            continue
        status |= check_script(path.name, module)
    verdict = 'a case is stale, or a script cannot be checked' if status else 'every case applies'
    print(f'{len(scripts)} mutation scripts checked: {verdict}')
    return status


if __name__ == '__main__':
    sys.exit(main())
