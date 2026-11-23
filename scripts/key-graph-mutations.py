"""Key-graph faults (phase 3 item 2): each must trip a fresh assertion in at least one of the test
targets it names; source bytes are always restored. python3 -I.

    python3 -I scripts/key-graph-mutations.py build/key-graph/mutations [case ...]
    python3 -I scripts/key-graph-mutations.py --check     # only that every location is still there

Run with no other Cargo or Gradle operation in this checkout, and do not read the source as the
truth while it runs: it is mutated then. A compilation failure never counts as caught: a Rust target
needs a "panicked at" line, a JVM target an assertion failure in a fresh test report. Every target
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
GRAPH_SRC = ROOT / 'lib/rust/crates/keygraph/src'
COMPUTE, CONTEXT, GRAPH_RS = GRAPH_SRC / 'compute.rs', GRAPH_SRC / 'context.rs', GRAPH_SRC / 'graph.rs'
ASSEMBLE, PARSE, FACTS, CALLS = (GRAPH_SRC / name for name in ('assemble.rs', 'parse.rs', 'facts.rs', 'calls.rs'))
ACTIONS = ROOT / 'lib/rust/crates/render/src/smartbar_actions.rs'
BRIDGE = ROOT / 'lib/rust/crates/awnative/src/render_smartbar.rs'
RENDER = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/render'
ROWS, KEYBOARD = RENDER / 'KeyGraphRows.kt', RENDER / 'RustKeyboard.kt'
PANELS = RENDER / 'RendererPanels.kt'
# The targets: Rust test binaries, and JVM test classes run by one Gradle call.
UNIT, BAR_TESTS = 'cargo:keygraph', 'cargo:smartbar_actions'
TRACE, BAR_MATCH = 'jvm:keygraph', 'jvm:smartbar'
GRAPH, BAR = [UNIT, TRACE], [BAR_TESTS, BAR_MATCH]
STAND_IN_KEY = '        if platform.misses() > asked {\n            return;\n        }\n        match computed'
STAND_IN_LABEL = '        if platform.misses() > asked {\n            return;\n        }\n        let group'
CASES = [
    # Computing one key's data: casing, currency slots, selectors, multi-text keys.
    ('auto-currency-cased', COMPUTE, 'Node::Auto(leaf, _) if eval.slots.is_some() && models::is_currency(leaf.code) => {',
     'Node::Auto(leaf, _) if false => {', GRAPH),
    ('case-ignores-shift', COMPUTE, 'let upper = models::select(0, eval.state, 3) != 0;', 'let upper = false;', GRAPH),
    ('selector-ignores-state', COMPUTE, 'models::select(*kind, eval.state, present)', 'models::select(*kind, 0, present)',
     GRAPH),
    ('multi-loses-its-id', COMPUTE, 'Node::Multi(leaf, id) => Some(leaf_data(leaf, Some(*id))),',
     'Node::Multi(leaf, _) => Some(leaf_data(leaf, None)),', GRAPH),
    ('missing-slot-kept', COMPUTE, 'let (code, label) = slots.get(at)?;',
     'let Some((code, label)) = slots.get(at) else { return Some(data) };', GRAPH),
    # What a key shows: icon, label, enabled state.
    ('undrawn-icon-kept', COMPUTE, 'if DRAWN.contains(&token) { token } else { 0 }', 'token', GRAPH),
    ('space-named-by-default', COMPUTE, '2 => Some(facts.language.clone()),', '2 => Some(facts.default_language.clone()),',
     GRAPH),
    ('mark-shown-plain', COMPUTE, 'models::text(data.code, &data.label, true, platform.mark(data.code))',
     'models::text(data.code, &data.label, true, false)', GRAPH),
    ('paste-ignores-the-lock', COMPUTE, 'codes::CLIPBOARD_PASTE => (bit(facts.locked), bit(!facts.locked && facts.pastable)),',
     'codes::CLIPBOARD_PASTE => (0, bit(facts.pastable)),', GRAPH + [BAR_MATCH]),
    # One key's computation: visibility, its popups and hints, and the order its questions come in.
    ('hidden-keys-shown', CONTEXT, 'match computed.filter(|data| compute::visible(self.facts, data.code)) {',
     'match computed {', GRAPH),
    ('own-popup-dropped', CONTEXT, 'if let Some(set) = data.popup {', 'if let Some(set) = data.popup.filter(|_| false) {',
     GRAPH),
    ('extended-in-every-mode', CONTEXT, 'if models::popup_mode(self.mode) {', 'if true {', GRAPH),
    ('label-not-lowercased', CONTEXT, 'let label = platform.text(ask::LOWER_TEXT, locale::SUBTYPE, &data.label);',
     'let label = data.label.clone();', GRAPH),
    ('group-ignored', CONTEXT, 'let extended: Vec<u16> = group.unwrap_or_else(|| label.clone());',
     'let extended: Vec<u16> = group.map_or_else(|| label.clone(), |_: Vec<u16>| label.clone());', GRAPH),
    ('key-specific-always', CONTEXT, 'if extended != label {', 'if true {', GRAPH),
    ('variation-ignored', CONTEXT, 'let variation = models::read(self.facts.state, 1).unwrap_or(0);',
     'let variation = 0;', GRAPH),
    ('hints-on-every-type', CONTEXT, 'if data.kind == CHARACTER {', 'if true {', GRAPH),
    ('hint-of-its-own-code', CONTEXT, 'hint.filter(|hint| hint.code != code)', 'hint.filter(|hint| hint.code != code || true)',
     GRAPH),
    ('hints-swapped', CONTEXT,
     'self.hint(key, data.code, symbol, Target::SymbolPopups, platform);\n'
     '            self.hint(key, data.code, number, Target::NumberPopups, platform);',
     'self.hint(key, data.code, number, Target::SymbolPopups, platform);\n'
     '            self.hint(key, data.code, symbol, Target::NumberPopups, platform);', GRAPH),
    ('hint-mapping-ignored', CONTEXT,
     'if let Some(set) = specific {\n            self.merge(&mut key.popups, set, target, platform);\n        }',
     'let _ = specific;', GRAPH),
    ('popup-main-dropped', CONTEXT, 'popups.merge(relevant, main, target);',
     'popups.merge(relevant, main.filter(|_| false), target);', GRAPH),
    ('fallback-ignored', CONTEXT, 'let mapping = if fallback {\n            self.fallback\n',
     'let mapping = if fallback {\n            None\n', GRAPH),
    ('always-enabled', CONTEXT, 'key.enabled = compute::enabled(self.facts, data.code);', 'key.enabled = true;', GRAPH),
    ('phone-hint-dropped', CONTEXT, 'key.hint = models::phone_hint(data.code).map(|hint| hint.encode_utf16().collect());',
     'key.hint = None;', GRAPH),
    ('space-hinted', CONTEXT, '} else if !data.is_space() || data.kind == NUMERIC {', '} else if true {', GRAPH),
    ('space-hint-shown', CONTEXT, '                .hint\n                .filter(|hint| !hint.is_space());',
     '                .hint;', GRAPH),
    ('stand-in-key-goes-on', CONTEXT, STAND_IN_KEY, '        match computed', GRAPH),
    ('stand-in-label-looked-up', CONTEXT, STAND_IN_LABEL, '        let group', GRAPH),
    ('misses-not-counted', FACTS, '        self.misses += 1;\n', '', GRAPH),
    # The graph: which keys are listed, the hints, the layout, the base codes, the smartbar's facts.
    ('listed-past-an-empty-row', GRAPH_RS, '            .take_while(|row| !row.is_empty())\n', '', GRAPH),
    ('hints-resolve-slots', GRAPH_RS, '            state: 0,\n            slots: None,',
     '            state: 0,\n            slots: Some(&[]),', GRAPH),
    ('hints-in-the-subtype-locale', GRAPH_RS, '            slots: None,\n            locale: locale::DEFAULT,',
     '            slots: None,\n            locale: locale::SUBTYPE,', GRAPH),
    ('layout-without-flex', GRAPH_RS, 'self.keys.iter().map(|key| key.flex).collect();',
     'self.keys.iter().map(|_| KeyFlex::default()).collect();', GRAPH),
    ('selector-base-code', GRAPH_RS, 'Node::Select(..) => 0,', 'Node::Select(..) => -1,', GRAPH),
    ('action-ignores-the-facts', GRAPH_RS, 'let facts = self.facts.clone().unwrap_or_default();',
     'let facts = Facts::default();', [UNIT, BAR_MATCH]),
    # Reading and merging the files.
    ('own-modifier-ignored', ASSEMBLE, '(Some(_), Some(own)) => own,', '(Some(_), Some(_)) => files.modifier,', GRAPH),
    ('placeholders-unseen', ASSEMBLE, 'Node::Text(leaf) => leaf.code,', 'Node::Text(_) => i32::MIN,', GRAPH),
    ('layout-parsed-twice', ASSEMBLE, '        if let Some(rows) = self.layouts.get(text) {\n            return rows.clone();\n        }\n',
     '', GRAPH),
    ('mapping-parsed-twice', ASSEMBLE,
     '        if let Some(mapping) = self.mappings.get(text) {\n            return mapping.clone();\n        }\n', '', GRAPH),
    ('unknown-class-refused', PARSE, '        _ => Node::Text(leaf(fields, arena, true)?),',
     '        _ => return Err(Error("an unknown class")),', GRAPH),
    ('key-type-case-sensitive', PARSE, 'let name = text(value)?.to_uppercase();', 'let name = text(value)?;', GRAPH),
    ('quoted-number-refused', PARSE, 'Value::String(text) => text.parse::<i64>().ok(),', 'Value::String(_) => None,', GRAPH),
    ('char-width-child-optional', PARSE, 'if !fields.contains_key(name) {', 'if false {', GRAPH),
    ('multi-code-from-the-file', PARSE, 'leaf.code = keys::codes::MULTIPLE_CODE_POINTS;', 'leaf.code = 0;', GRAPH),
    # The answers Android gives.
    ('mark-asked-below-the-first', FACTS, 'if code < FIRST_MARK {', 'if code < 0 {', GRAPH),
    ('code-answer-ignored', FACTS, '.insert((question.ask, question.locale, code), answer);',
     '.insert((question.ask, question.locale, code), {\n                    let _: i32 = answer;\n                    code\n                });',
     GRAPH),
    ('text-answer-ignored', FACTS, 'return answer.clone();', 'return text.to_vec();', GRAPH),
    # The rows that cross to Android.
    ('nothing-prioritized', CALLS, 'let mut out = vec![text(keys.prioritized.len())];', 'let mut out = vec![text(0)];', GRAPH),
    ('base-code-computed', CALLS, 'row[key::BASE] = text(base);',
     'row[key::BASE] = text(if base == 0 { 0 } else { key.data.code });', GRAPH),
    ('enabled-flag-from-visible', CALLS, 'row[key::ENABLED] = wire::flag::utf16(key.enabled);',
     'row[key::ENABLED] = wire::flag::utf16(key.visible);', GRAPH),
    # The Kotlin that reads the files in, reports the facts, answers, and reads the keys back.
    ('no-symbols-for-hints', ROWS, 'if (layoutMode != mode && mode != KeyboardMode.CHARACTERS) continue',
     'if (layoutMode != mode) continue', [TRACE]),
    ('own-modifier-not-named', ROWS, '        if (own != null) loaded = loaded or set.names\n', '', [TRACE]),
    ('fallback-not-sent', ROWS, '    put(KeyGraphFile.FALLBACK, KeyGraphBuild.FALLBACK, files.fallback)\n', '', [TRACE]),
    ('slot-labels-lost', ROWS, 'slots.flatMap { listOf("${it.code}", it.label) }', 'slots.flatMap { listOf("${it.code}", "") }',
     [TRACE]),
    ('upper-code-lowered', ROWS, 'KeyGraphAsk.UPPER_CODE -> "${codeText(value.toInt()).uppercase(locale)',
     'KeyGraphAsk.UPPER_CODE -> "${codeText(value.toInt()).lowercase(locale)', [TRACE]),
    ('text-cased-in-root', ROWS, 'KeyGraphAsk.LOWER_TEXT -> value.lowercase(locale)',
     'KeyGraphAsk.LOWER_TEXT -> value.lowercase(Locale.ROOT)', [TRACE]),
    ('merge-flag-lost', KEYBOARD, 'KeyGraphHints.MERGE to WireFlag.encode(config.mergeHintPopups),',
     'KeyGraphHints.MERGE to WireFlag.encode(false),', [TRACE]),
    ('nothing-prioritized-read', KEYBOARD, 'return PopupKeys(null, all.take(prioritized), all.drop(prioritized))',
     'return PopupKeys(null, all.take(prioritized * 0), all.drop(prioritized * 0))', [TRACE]),
    ('popup-set-per-read', KEYBOARD, 'popupSets.getOrPut(it) { PopupSet() }', 'PopupSet<AbstractKeyData>()', [TRACE]),
    ('multi-per-read', KEYBOARD, 'if (multi >= 0) return multis.getOrPut(multi) {', 'if (multi >= 0) return run {', [TRACE]),
    ('label-flag-ignored', KEYBOARD, 'label = row[at + KeyGraphKey.SHOWN].takeIf { WireFlag.decode(row[at + KeyGraphKey.LABELED]) }',
     'label = row[at + KeyGraphKey.SHOWN]', [TRACE]),
    ('one-round-of-answers', KEYBOARD, 'while (questions.isNotEmpty()) {', 'if (questions.isNotEmpty()) {', [TRACE]),
    # The smartbar's questions answered from the graph.
    ('key-icon-on-every-action', ACTIONS, 'let key_icon = if action.icon == icon::KEY {', 'let key_icon = if true {',
     BAR),
    ('key-label-ignored', ACTIONS, 'row[shown::LABEL] = if action.key_label {', 'row[shown::LABEL] = if false {', BAR),
    ('empty-slot-asked', ACTIONS, 'let answer = if kind == kind::NONE {', 'let answer = if false {', BAR),
    ('key-icon-flag-ignored', ACTIONS, 'key_icon: kind == kind::KEY && answer.icon != 0,', 'key_icon: kind == kind::KEY,',
     BAR),
    ('bridge-enables-every-key', BRIDGE, 'enabled: action.enabled,', 'enabled: true,', [BAR_MATCH]),
    ('key-icon-token-lost', ACTIONS, 'text(icons::smartbar_token(action.icon, key_icon))',
     'text(icons::smartbar_token(action.icon, 0))', BAR),
    # The overflow and editor panels' questions about each action's key.
    ('action-always-enabled', CALLS, 'answered[action_answer::ENABLED] = wire::flag::utf16(answer.enabled);',
     'answered[action_answer::ENABLED] = wire::flag::utf16(true);', [UNIT, BAR_MATCH]),
    ('action-always-labeled', CALLS, 'answered[action_answer::LABELED] = wire::flag::utf16(answer.label.is_some());',
     'answered[action_answer::LABELED] = wire::flag::utf16(true);', [UNIT, BAR_MATCH]),
    ('panel-type-dropped', PANELS, 'KeyGraphAction.TYPE to "${key.type.ordinal}",', 'KeyGraphAction.TYPE to "0",',
     [BAR_MATCH]),
    ('panel-icon-lost', PANELS, 'fun icon(i: Int): Int = field(i, KeyGraphActionAnswer.ICON).toInt()',
     'fun icon(i: Int): Int = field(i, KeyGraphActionAnswer.ICON).toInt() * 0', [BAR_MATCH]),
    ('panel-label-flag-misread', PANELS, 'takeIf { WireFlag.decode(field(i, KeyGraphActionAnswer.LABELED)) }',
     'takeIf { WireFlag.decode(field(i, KeyGraphActionAnswer.ENABLED)) }', [BAR_MATCH]),
]
JVM = {TRACE: ['*KeyGraphMatchTest'], BAR_MATCH: ['*SmartbarActionsMatchTest']}
CARGO = {UNIT: ['-p', 'keygraph'], BAR_TESTS: ['-p', 'render', '--test', 'smartbar_actions']}
REPORTS = ROOT / 'app/build/test-results/testDebugUnitTest'


def env():
    e = dict(os.environ)
    e.setdefault('CARGO_BUILD_JOBS', '6')
    e.setdefault('ANDROID_HOME', str(Path.home() / 'Library/Android/sdk'))
    return e


def log_of(output, name, target):
    return output / (name + '.' + target.replace(':', '-') + '.log')


def compiled(log_path):
    """False when the build failed: a mutant that doesn't compile is no test's catch."""
    text = log_path.read_text(errors='replace')
    return not any(sign in text for sign in ('could not compile', '\ne: ', 'Compilation error'))


def rust(output, name, target):
    log_path = log_of(output, name, target)
    with log_path.open('wb') as log:
        result = subprocess.run(['cargo', 'test', '-q'] + CARGO[target], cwd=ROOT / 'lib/rust', stdout=log,
                                stderr=subprocess.STDOUT, env=env())
    text = log_path.read_text(errors='replace')
    return result.returncode, sum('panicked at' in line for line in text.splitlines())


def assertion(failure):
    kind = failure.attrib.get('type', '')
    return 'AssertionFailedError' in kind or kind == 'java.lang.AssertionError'


def jvm(output, name, target):
    started = time.time()
    command = ['./gradlew', '--stacktrace', ':app:testDebugUnitTest']
    for pattern in JVM[target]:
        command += ['--tests', pattern]
    with log_of(output, name, target).open('wb') as log:
        result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env())
    failures = 0
    for report in REPORTS.glob('TEST-*.xml'):
        if report.stat().st_mtime >= started:
            failures += sum(assertion(f) for f in ET.parse(report).getroot().iter('failure'))
    return result.returncode, failures


def run(output, name, target):
    return (rust if target.startswith('cargo:') else jvm)(output, name, target)


def check(cases):
    originals = {path: path.read_bytes() for _, path, _, _, _ in cases}
    for name, path, old, new, _ in cases:
        assert originals[path].decode().count(old) == 1, 'mutation location changed: ' + name
        assert old != new, 'mutation changes nothing: ' + name
    names = [case[0] for case in cases]
    assert len(names) == len(set(names)), 'case names repeat'
    return originals


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
    originals = check(cases)
    output = ROOT / args.output
    output.mkdir(parents=True, exist_ok=False)
    results = []
    try:
        for name, path, old, new, targets in cases:
            path.write_text(originals[path].decode().replace(old, new))
            try:
                outcome = {}
                for target in targets:
                    status, failures = run(output, name, target)
                    outcome[target] = {'exit': status, 'assertion_failures': failures,
                                       'compiled': compiled(log_of(output, name, target))}
            finally:
                path.write_bytes(originals[path])
            caught = [t for t, o in outcome.items() if o['exit'] != 0 and o['assertion_failures'] > 0]
            results.append({'mutation': name, 'file': str(path.relative_to(ROOT)), 'targets': outcome,
                            'caught_by': caught})
            (output / 'result.json').write_text(json.dumps({'mutations': results}, indent=2) + '\n')
            broken = [t for t, o in outcome.items() if not o['compiled']]
            missed = 'NOT CAUGHT' + (' (did not compile for ' + ', '.join(broken) + ')' if broken else '')
            print(name + ': ' + ('caught by ' + ', '.join(caught) if caught else missed), flush=True)
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
