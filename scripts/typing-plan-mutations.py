"""Typing-plan and smartbar-plan faults (phase 3 item 1): each must trip a fresh assertion in at least
one of the test targets it names; source bytes are always restored. python3 -I.

    python3 -I scripts/typing-plan-mutations.py build/typing-fan-out/mutations [case ...]

Run with no other Cargo or Gradle operation in this checkout, and do not read the source as the
truth while it runs: it is mutated then. A compilation failure never counts as caught: a Rust target
needs a "panicked at" line, a JVM target an AssertionFailedError in a fresh test report. Every target
a case names is run and recorded, so the result says which caught it.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

sys.dont_write_bytecode = True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

ROOT = Path(__file__).resolve().parent.parent
TYPING = ROOT / 'lib/rust/crates/nlp/src/typing.rs'
COUNTERS = ROOT / 'lib/rust/crates/nlp/src/counters.rs'
ACTIONS = ROOT / 'lib/rust/crates/render/src/smartbar_actions.rs'
ICONS = ROOT / 'lib/rust/crates/icons/src/sites.rs'
NLP = ROOT / 'app/src/main/kotlin/dev/patrickgold/florisboard/ime/nlp'
EVENTS, STATE = NLP / 'TypingEvents.kt', NLP / 'NlpState.kt'
SMARTBAR = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/render/RendererSmartbar.kt'
# The targets: Rust test binaries, and JVM test classes run by one Gradle call.
NLP_TESTS, BAR_TESTS = 'cargo:nlp', 'cargo:smartbar_actions'
TRACE, BAR_MATCH = 'jvm:typing', 'jvm:smartbar'
PLAN, BAR = [NLP_TESTS, TRACE], [BAR_TESTS, BAR_MATCH]
CASES = [
    # The typing plan: what each event counts, learns and teaches, and what it asks of Android.
    ('arm-only-when-accepted', TYPING, 'if flag_at(f, row::COMMIT) && flag_at(f, row::CORRECTS) {',
     'if plan.accepted && flag_at(f, row::COMMIT) && flag_at(f, row::CORRECTS) {', PLAN),
    ('arm-an-unchanged-word', TYPING, 'if !typed.is_empty() && start >= 0 && corrected != typed {',
     'if !typed.is_empty() && start >= 0 {', PLAN),
    ('undo-with-a-selection', TYPING, 'if !flag_at(f, row::CURSOR_MODE) {', 'if false {', PLAN),
    ('kept-even-when-not-remembered', TYPING, 's.kept.add(&armed.typed, self.context.remember_kept())',
     's.kept.add(&armed.typed, true)', PLAN),
    ('backspace-keeps-the-swipe', TYPING, 'event::BACKSPACE => {\n                s.learner.drop_swiped();\n',
     'event::BACKSPACE => {\n', PLAN),
    ('learned-on-removal', TYPING, 'if word && flag_at(f, row::COMMIT) {', 'if word {', PLAN),
    ('picked-for-the-wrong-word', TYPING, 's.taps.picked(!f[row::TYPED].is_empty());',
     's.taps.picked(f[row::TYPED].is_empty());', PLAN),
    ('second-save-lost', TYPING, 'number(self.saves.get(1).copied().unwrap_or(-1))', 'number(-1)', PLAN),
    ('undone-always-counted', COUNTERS, 'if self.counting() {\n            self.counters.undone();',
     'if true {\n            self.counters.undone();', PLAN),
    ('field-keeps-the-trail', TYPING, '                s.learner.reset();\n                s.taps.clear();\n',
     '                s.learner.reset();\n', PLAN),
    ('learned-without-the-dictionary', TYPING, 'if row.is_empty() || !self.learns(lookup.is_some()) {',
     'if row.is_empty() || !self.learns(true) {', PLAN),
    ('pick-not-published', TYPING, 'publish: with_context(kind) && kind != event::UNDO_CORRECTION,',
     'publish: with_context(kind) && kind != event::UNDO_CORRECTION && kind != event::PICKED,', PLAN),
    ('undo-not-published', TYPING, '        plan.publish = true;\n        plan.kept =', '        plan.kept =', PLAN),
    ('aim-setting-ignored', TYPING, 'if self.aim(f) {', 'if { self.aim(f); true } {', PLAN),
    ('typed-not-taught', TYPING, '                s.taps.typed(&f[row::WORD]);\n', '', PLAN),
    ('key-not-counted', TYPING, 'event::KEY => self.counters.key(),', 'event::KEY => {}', PLAN),
    ('pick-never-saves', TYPING,
     '        self.check(s.taps, int(f, row::NOW), plan);\n        if word && flag_at(f, row::COMMIT) {',
     '        if word && flag_at(f, row::COMMIT) {', PLAN),
    # The Kotlin that carries each event in and does what its plan asks.
    ('kept-not-written', EVENTS, '        if (WireFlag.decode(plan[NlpEventPlan.KEPT])) saveKept()\n', '', [TRACE]),
    ('second-timer-dropped', EVENTS, '        saveAfter(plan[NlpEventPlan.SAVE_AGAIN].toLong())\n', '', [TRACE]),
    ('typed-word-lost', EVENTS, 'NlpEvent.TYPED to content.composingText', 'NlpEvent.TYPED to ""', [TRACE]),
    ('aim-setting-lost', EVENTS, '(NlpEvent.AIM to WireFlag.encode(aim))', '(NlpEvent.AIM to WireFlag.encode(true))',
     [TRACE]),
    ('dictionary-never-lent', EVENTS, 'candidate, content != null)', 'candidate, false)', [TRACE]),
    ('candidate-id-lost', STATE, 'if (candidate != null) event[NlpEvent.CANDIDATE] =',
     'if (false) event[NlpEvent.CANDIDATE] =', [TRACE]),
    ('plan-not-published', STATE, 'if (WireFlag.decode(plan[NlpEventPlan.PUBLISH])) publish(',
     'if (false) publish(', [TRACE]),
    # The smartbar plan: each action's icon, label and flags, and what the toggle flips.
    ('arrow-ignores-flip', ACTIONS, 'if self.flip ^ (self.expanded() && self.incognito_display == BEHIND_KEYS) {',
     'if self.expanded() && self.incognito_display == BEHIND_KEYS {', BAR),
    ('arrow-ignores-expansion', ACTIONS,
     'if self.flip ^ (self.expanded() && self.incognito_display == BEHIND_KEYS) {', 'if self.flip {', BAR),
    ('incognito-in-any-mode', ACTIONS, '} else if self.incognito && self.incognito_display == REPLACES_TOGGLE {',
     '} else if self.incognito {', BAR),
    ('unfold-swapped', ACTIONS, 'if self.extended {\n                icon::UNFOLD_LESS',
     'if !self.extended {\n                icon::UNFOLD_LESS', BAR),
    ('empty-slot-disabled', ACTIONS, 'if action.kind == kind::NONE || action.enabled {', 'if action.enabled {', BAR),
    ('level-unclamped', ACTIONS, 'self.level.clamp(0.0, 1.0)', 'self.level.max(0.0)', BAR),
    ('finishing-while-listening', ACTIONS, 'if !self.listening {', 'if self.dictating {', BAR),
    ('stop-never-shown', ACTIONS, 'if code(action) == READ_ALOUD && self.reading {\n            icon::STOP',
     'if false {\n            icon::STOP', BAR),
    ('half-a-surrogate-pair', ACTIONS, 'text.len().min(if paired { 2 } else { 1 })', 'text.len().min(1)', BAR),
    ('blank-text-shown', ACTIONS, 'if chars::is_blank(first) {', 'if first.is_empty() {', BAR),
    ('key-label-under-its-icon', ACTIONS, 'key_label: action.kind == kind::KEY && icon == icon::NONE,',
     'key_label: action.kind == kind::KEY,', BAR),
    ('preview-turns-the-arrow', ACTIONS, 'shared: config[SHARED_SETTING] != 0.0,', 'shared: c.shared,', BAR),
    ('toggle-flips-shared-in-extended', ACTIONS, 'toggles_extended: bar.extended_layout(),',
     'toggles_extended: false,', BAR),
    # The icon each action draws. This was Kotlin in RendererSmartbar.kt until the keyboard sent icon tokens.
    ('arrow-vector-swapped', ICONS, 'icon::ARROW_LEFT => ARROW_LEFT,', 'icon::ARROW_LEFT => ARROW_RIGHT,', BAR),
    # What the rows carry from each key. This was Kotlin in RendererSmartbar.kt until its keys came from the key graph.
    ('key-label-dropped', ACTIONS, 'answer.label.clone().unwrap_or_default()', 'Vec::new()', BAR),
    ('disabled-keys-enabled', ACTIONS, 'enabled: answer.enabled,', 'enabled: true,', BAR),
    # The Kotlin that tells the smartbar what dictation is doing.
    ('listening-when-finishing', SMARTBAR, 'LISTENING to (dictation == Dictation.Status.LISTENING)',
     'LISTENING to (dictation != Dictation.Status.IDLE)', [BAR_MATCH]),
]
JVM = {TRACE: ['*TypingTraceMatchTest', '*TypingEventsTest'], BAR_MATCH: ['*SmartbarActionsMatchTest']}
REPORTS = ROOT / 'app/build/test-results/testDebugUnitTest'


def env():
    e = dict(os.environ)
    e.setdefault('CARGO_BUILD_JOBS', '6')
    e.setdefault('ANDROID_HOME', str(Path.home() / 'Library/Android/sdk'))
    return e


def rust(output, name, target):
    package, test = ('nlp', None) if target == NLP_TESTS else ('render', 'smartbar_actions')
    command = ['cargo', 'test', '-q', '-p', package] + (['--test', test] if test else [])
    log_path = output / (name + '.' + target.replace(':', '-') + '.log')
    with log_path.open('wb') as log:
        result = subprocess.run(command, cwd=ROOT / 'lib/rust', stdout=log, stderr=subprocess.STDOUT, env=env())
    text = log_path.read_text(errors='replace')
    return result.returncode, sum('panicked at' in line for line in text.splitlines())


def jvm(output, name, target):
    started = time.time()
    command = ['./gradlew', '--stacktrace', ':app:testDebugUnitTest']
    for pattern in JVM[target]:
        command += ['--tests', pattern]
    with (output / (name + '.' + target.replace(':', '-') + '.log')).open('wb') as log:
        result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env())
    failures = 0
    for report in REPORTS.glob('TEST-*.xml'):
        if report.stat().st_mtime >= started:
            failures += sum('AssertionFailedError' in f.attrib.get('type', '')
                            for f in ET.parse(report).getroot().iter('failure'))
    return result.returncode, failures


def run(output, name, target):
    return (rust if target.startswith('cargo:') else jvm)(output, name, target)


def check_edits():
    return mutation_cases.rows(CASES)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('cases', nargs='*', help='case names to run (default: all)')
    args = parser.parse_args()
    unknown = set(args.cases) - {case[0] for case in CASES}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    cases = [case for case in CASES if not args.cases or case[0] in args.cases]
    output = ROOT / args.output
    output.mkdir(parents=True, exist_ok=False)
    originals = {path: path.read_bytes() for _, path, _, _, _ in cases}
    for name, path, old, _, _ in cases:
        assert originals[path].decode().count(old) == 1, 'mutation location changed: ' + name
    results = []
    try:
        for name, path, old, new, targets in cases:
            path.write_text(originals[path].decode().replace(old, new))
            try:
                outcome = {}
                for target in targets:
                    status, failures = run(output, name, target)
                    outcome[target] = {'exit': status, 'assertion_failures': failures}
            finally:
                path.write_bytes(originals[path])
            caught = [t for t, o in outcome.items() if o['exit'] != 0 and o['assertion_failures'] > 0]
            results.append({'mutation': name, 'file': str(path.relative_to(ROOT)), 'targets': outcome,
                            'caught_by': caught})
            (output / 'result.json').write_text(json.dumps({'mutations': results}, indent=2) + '\n')
            print(name + ': ' + ('caught by ' + ', '.join(caught) if caught else 'NOT CAUGHT'), flush=True)
    finally:
        for path, original in originals.items():
            path.write_bytes(original)
    restored = {}
    for target in sorted({t for case in cases for t in case[4]}):
        status, failures = run(output, 'restored', target)
        restored[target] = {'exit': status, 'assertion_failures': failures}
    missed = [r['mutation'] for r in results if not r['caught_by']]
    report = {'mutations': results, 'missed': missed, 'restored': restored,
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in originals}}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    assert all(r['exit'] == 0 and r['assertion_failures'] == 0 for r in restored.values()), 'restored source fails'
    assert not missed, f'not caught: {missed}'
    print(f'All {len(results)} mutations caught; restored source passes {sorted(restored)}', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
