"""Subtype faults (phase 3 item 3, the subtype half): each must trip a fresh assertion in at least
one of the test targets it names; source bytes are always restored. python3 -I.

    python3 -I scripts/subtype-mutations.py build/subtype/mutations [case ...]
    python3 -I scripts/subtype-mutations.py --check     # only that every location is still there

The runner is theme-subtype-mutations.py's, with the same rules: run with no other Cargo or Gradle
operation in this checkout, a compilation failure never counts as caught, and every target a case
names is run and recorded. The trace target is SubtypeMatchTest (3 seeds x 50,000 states against
main's frozen subtypes and build row) with SubtypeSessionTest.

Left out as equivalent: not naming the default subtype's tags (its only tag, en-US, is already
Android's name for it), and storing a switch by id before or after making it active (the store
never suspends, so nothing can tell the orders apart).
"""
import importlib.util
import sys
from pathlib import Path

# Loading the runner and mutation_cases as modules would otherwise leave a __pycache__ in scripts/.
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

SPEC = importlib.util.spec_from_file_location('theme_subtype_mutations', Path(__file__).parent / 'theme-subtype-mutations.py')
RUNNER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNNER)

ROOT = RUNNER.ROOT
SRC = ROOT / 'lib/rust/crates/imesubtype/src'
SESSION, CALLS = SRC / 'session.rs', SRC / 'calls.rs'
DECODE = ROOT / 'lib/rust/crates/settings/src/subtype.rs'
CORE = ROOT / 'app/src/main/kotlin/dev/patrickgold/florisboard/ime/core'
KOTLIN_SESSION, MANAGER = CORE / 'SubtypeSession.kt', CORE / 'SubtypeManager.kt'
ROWS = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/render/KeyGraphRows.kt'
# The targets: the Rust crates' tests, and the JVM subtype tests run by one Gradle call.
UNIT, SETTINGS, TRACE = 'cargo:imesubtype', 'cargo:settings', 'jvm:subtype'
BOTH = [UNIT, TRACE]
CASES = [
    # What the session decides: repairs, the active subtype, the language key, switches, keys.
    ('repair-not-reported', SESSION, 'changed |= named != new;', 'changed |= false;', BOTH),
    ('stored-id-always-written', SESSION, 'write: id != stored,', 'write: true,', BOTH),
    ('stored-id-never-written', SESSION, 'write: id != stored,', 'write: false,', BOTH),
    ('evaluate-ignores-stored', SESSION, 'subtype::active(&ids, stored, -1)', 'subtype::active(&ids, -1, -1)', BOTH),
    ('cycle-without-the-active', SESSION, '.map(|v| i64::from(*v == self.active))', '.map(|_| 0_i64)', BOTH),
    ('select-last-match', SESSION, '.position(|&i| i == id) else {', '.rposition(|&i| i == id) else {', BOTH),
    ('select-never-stored', SESSION, 'Ok(Some(Switch { write: true, id }))', 'Ok(Some(Switch { write: false, id }))', BOTH),
    ('every-subtype-one-key', SESSION, '*self.keys.entry(value.clone()).or_insert(next)',
     '*self.keys.entry(value.clone()).or_insert(next) * 0', BOTH),
    ('tags-left-unnamed', SESSION, 'self.names.get(tag).cloned().unwrap_or_else(|| tag.to_vec())', 'tag.to_vec()', BOTH),
    ('known-tags-asked-again', SESSION, 'if !self.names.contains_key(tag) && !asked', 'if !asked', BOTH),
    ('names-misaligned', SESSION, '.into_iter().zip(names)', '.into_iter().zip(names.iter().rev())', BOTH),
    # The rows that cross to Android.
    ('repaired-list-empty', CALLS, 'let list = Value::List(session.list().to_vec());', 'let list = Value::List(vec![]);',
     BOTH),
    ('next-is-previous', CALLS, 'let switch = session.cycle(false)?;', 'let switch = session.cycle(true)?;', BOTH),
    ('active-asks-a-write', CALLS, 'Some(Switch { write: false, id })', 'Some(Switch { write: true, id })', BOTH),
    ('characters-and-symbols-swapped', CALLS, '("characters", subtype::CHARACTERS),\n    ("symbols", subtype::SYMBOLS),',
     '("characters", subtype::SYMBOLS),\n    ("symbols", subtype::CHARACTERS),', BOTH),
    ('suggestion-from-spelling', CALLS, 'providers.get("suggestion")?', 'providers.get("spelling")?', BOTH),
    ('popup-from-punctuation', CALLS, 'out[subtype::POPUP] = text("popupMapping")?;',
     'out[subtype::POPUP] = text("punctuationRule")?;', BOTH),
    ('currency-index-off-by-one', CALLS, 'index.map_or(-1, |i| i as i64)', 'index.map_or(-1, |i| i as i64 + 1)', BOTH),
    ('secondary-count-zero', CALLS, 'out[subtype::SECONDARY] = number_text(secondary.len());',
     'out[subtype::SECONDARY] = number_text(0);', BOTH),
    # The settings codec's reading of a blank list.
    ('blank-list-refused', DECODE, 'return Ok(Value::List(vec![]));', 'return Err("blank");', [SETTINGS, TRACE]),
    # The Kotlin that names tags, reads the rows and reports the currency sets.
    ('tags-named-as-given', KOTLIN_SESSION, '*tags.map(name).toTypedArray()', '*tags', [TRACE]),
    ('repair-dropped', KOTLIN_SESSION, '.takeIf { WireFlag.decode(rows[ImeSubtypeLoaded.CHANGED]) }', '.takeIf { false }',
     [TRACE]),
    ('currency-sets-reported-once', KOTLIN_SESSION, 'if (sets !== reportedMap) {',
     'if (sets !== reportedMap && reportedMap == null) {', [TRACE]),
    ('every-layout-the-characters', KOTLIN_SESSION, 'field(CHARACTERS + it.ordinal)', 'field(CHARACTERS)', [TRACE]),
    ('facts-popup-from-composer', KOTLIN_SESSION, 'layouts, subtype.popupMapping)', 'layouts, subtype.composer)', [TRACE]),
    ('key-from-id', KOTLIN_SESSION, 'SubtypeFacts(field(KEY).toInt(),', 'SubtypeFacts(field(ID).toLong().toInt(),', [TRACE]),
    ('switch-always-stored', KOTLIN_SESSION, 'Switched(active, WireFlag.decode(rows[ImeSubtypeSwitched.WRITE]))',
     'Switched(active, true)', [TRACE]),
    # The manager: what it stores and publishes.
    ('evaluate-not-stored', MANAGER,
     'session.evaluate(store.activeId())\n        if (switched.write) store.setActiveId(switched.active.subtype.id)',
     'session.evaluate(store.activeId())', [TRACE]),
    ('language-key-not-stored', MANAGER,
     'val switched = direction()\n        if (switched.write) store.setActiveId(switched.active.subtype.id)',
     'val switched = direction()', [TRACE]),
    ('repaired-list-not-stored', MANAGER, '            loaded.repaired?.let { scope.launch { store.setList(it) } }\n', '',
     [TRACE]),
    ('not-evaluated-on-load', MANAGER, '            evaluateActiveSubtype()\n', '', [TRACE]),
    ('currency-sets-ignored', MANAGER, 'session.currencySet(currencySets(), subtypeToSearch.currencySet)',
     'session.currencySet(null, subtypeToSearch.currencySet)', [TRACE]),
    # The key graph: the popup mapping it reads and the cache key it keeps keyboards under.
    ('popup-mapping-always-default', ROWS, 'mapping(facts.popupMapping), mapping(extCorePopupMapping("default")))',
     'mapping(extCorePopupMapping("default")), mapping(extCorePopupMapping("default")))', [TRACE]),
    ('keyboards-by-mode-only', ROWS, 'get(mode.value, facts.key) ?: build().also { set(mode.value, facts.key, it) }',
     'get(mode.value, 0) ?: build().also { set(mode.value, 0, it) }', [TRACE]),
]
RUNNER.CARGO.update({UNIT: ['-p', 'imesubtype'], SETTINGS: ['-p', 'settings']})
RUNNER.JVM[TRACE] = (':app:testDebugUnitTest', ['*SubtypeSessionTest', '*SubtypeMatchTest'],
                     ROOT / 'app/build/test-results/testDebugUnitTest')


def check_edits():
    return mutation_cases.rows(CASES)


if __name__ == '__main__':
    mutation_cases.check_flag()
    RUNNER.main(CASES, __doc__)
