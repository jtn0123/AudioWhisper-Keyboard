"""Typing events through the NLP session (TypingTraceMatchTest): typing-plan-mutations.py's cases
for the typing plan and the Kotlin that carries each event, run against the trace alone. That script
also runs them against the nlp crate's own tests, and its smartbar cases guard a render reference,
which stays."""
TYPING = 'lib/rust/crates/nlp/src/typing.rs'
COUNTERS = 'lib/rust/crates/nlp/src/counters.rs'
NLP = 'app/src/main/kotlin/dev/patrickgold/florisboard/ime/nlp/'
EVENTS, STATE = NLP + 'TypingEvents.kt', NLP + 'NlpState.kt'
MODULE = ':app'
TARGETS = {
    'trace': ['*TypingTraceMatchTest'],
    'tap': ['*StripTapTest', '*TypingTraceMatchTest'],
}
CASES = [
    # The typing plan: what each event counts, learns and teaches, and what it asks of Android.
    ('arm-only-when-accepted', TYPING, 'if flag_at(f, row::COMMIT) && flag_at(f, row::CORRECTS) {',
     'if plan.accepted && flag_at(f, row::COMMIT) && flag_at(f, row::CORRECTS) {', 'trace'),
    ('arm-an-unchanged-word', TYPING, 'if !typed.is_empty() && start >= 0 && corrected != typed {',
     'if !typed.is_empty() && start >= 0 {', 'trace'),
    ('undo-with-a-selection', TYPING, 'if !flag_at(f, row::CURSOR_MODE) {', 'if false {', 'trace'),
    ('kept-even-when-not-remembered', TYPING, 's.kept.add(&armed.typed, self.context.remember_kept())',
     's.kept.add(&armed.typed, true)', 'trace'),
    ('backspace-keeps-the-swipe', TYPING, 'event::BACKSPACE => {\n                s.learner.drop_swiped();\n',
     'event::BACKSPACE => {\n', 'trace'),
    ('learned-on-removal', TYPING, 'if word && flag_at(f, row::COMMIT) {', 'if word {', 'trace'),
    ('picked-for-the-wrong-word', TYPING, 's.taps.picked(!f[row::TYPED].is_empty());',
     's.taps.picked(f[row::TYPED].is_empty());', 'trace'),
    ('second-save-lost', TYPING, 'number(self.saves.get(1).copied().unwrap_or(-1))', 'number(-1)', 'trace'),
    ('undone-always-counted', COUNTERS, 'if self.counting() {\n            self.counters.undone();',
     'if true {\n            self.counters.undone();', 'trace'),
    ('field-keeps-the-trail', TYPING, '                s.learner.reset();\n                s.taps.clear();\n',
     '                s.learner.reset();\n', 'trace'),
    ('learned-without-the-dictionary', TYPING, 'if row.is_empty() || !self.learns(lookup.is_some()) {',
     'if row.is_empty() || !self.learns(true) {', 'trace'),
    ('pick-not-published', TYPING, 'publish: with_context(kind) && kind != event::UNDO_CORRECTION,',
     'publish: with_context(kind) && kind != event::UNDO_CORRECTION && kind != event::PICKED,', 'trace'),
    ('undo-not-published', TYPING, '        plan.publish = true;\n        plan.kept =', '        plan.kept =', 'trace'),
    ('aim-setting-ignored', TYPING, 'if self.aim(f) {', 'if { self.aim(f); true } {', 'trace'),
    ('typed-not-taught', TYPING, '                s.taps.typed(&f[row::WORD]);\n', '', 'trace'),
    ('key-not-counted', TYPING, 'event::KEY => self.counters.key(),', 'event::KEY => {}', 'trace'),
    ('pick-never-saves', TYPING,
     '        self.check(s.taps, int(f, row::NOW), plan);\n        if word && flag_at(f, row::COMMIT) {',
     '        if word && flag_at(f, row::COMMIT) {', 'trace'),
    # The Kotlin that carries each event in and does what its plan asks.
    ('kept-not-written', EVENTS, '        if (WireFlag.decode(plan[NlpEventPlan.KEPT])) saveKept()\n', '', 'trace'),
    ('second-timer-dropped', EVENTS, '        saveAfter(plan[NlpEventPlan.SAVE_AGAIN].toLong())\n', '', 'trace'),
    ('typed-word-lost', EVENTS, 'NlpEvent.TYPED to content.composingText', 'NlpEvent.TYPED to ""', 'trace'),
    ('aim-setting-lost', EVENTS, '(NlpEvent.AIM to WireFlag.encode(aim))', '(NlpEvent.AIM to WireFlag.encode(true))',
     'trace'),
    ('dictionary-never-lent', EVENTS, 'candidate, content != null)', 'candidate, false)', 'trace'),
    ('candidate-id-lost', STATE, 'if (candidate != null) event[NlpEvent.CANDIDATE] =',
     'if (false) event[NlpEvent.CANDIDATE] =', 'trace'),
    ('plan-not-published', STATE, 'if (WireFlag.decode(plan[NlpEventPlan.PUBLISH])) publish(',
     'if (false) publish(', 'trace'),
    # A tap on a row the StateFlow kept from an equal list: refused again, or found by value, which
    # takes the first of two equal words for the one tapped.
    ('stale-tap-refused', STATE, 'shownIds.getOrNull(candidatesMutable.value.indexOfFirst { it === candidate })',
     'shownIds.getOrNull(-1)', 'tap'),
    ('stale-tap-by-value', STATE, 'candidatesMutable.value.indexOfFirst { it === candidate }',
     'candidatesMutable.value.indexOfFirst { it == candidate }', 'tap'),
]
