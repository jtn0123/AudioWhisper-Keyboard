"""Dictation's rules and mic key (DictationRulesMatchTest, MicKeyMatchTest): no other script mutates
them. The session, the microphone and the model check are covered by dictation-mutations.py (state)
and edges-mutations.py --part dictation."""
DICT = 'lib/rust/crates/dictation/src/'
GLUE = 'lib/rust/crates/awnative/src/dictation.rs'
MODULE = ':app'
TARGETS = {'rules': ['*DictationRulesMatchTest'], 'mic-key': ['*MicKeyMatchTest']}
CASES = [
    # Joining phrases: the full stop taken back, the capital kept or lowered, the space between.
    ('stop-taken-without-letters', DICT + 'phrases.rs', '            && last.iter().any(|&u| is_letter_or_digit(u))\n',
     '', 'rules'),
    ('acronym-lowered', DICT + 'phrases.rs', '        || (word.len() > 1 && is_upper_case(word[1]))\n', '', 'rules'),
    ('semicolon-ends-the-sentence', DICT + 'phrases.rs', 'is_letter_or_digit(u) || u == unit(\',\') || u == unit(\';\')',
     'is_letter_or_digit(u) || u == unit(\',\')', 'rules'),
    ('right-quote-ends-a-word', DICT + 'phrases.rs', "u == unit('\\'') || u == RIGHT_QUOTE)",
     "u == unit('\\'') || u == RIGHT_QUOTE + 1)", 'rules'),
    ('comma-not-replaced', DICT + 'phrases.rs', 'last.last().is_some_and(|u| MARKS.contains(u)) &&',
     'last.last().is_some_and(|u| MARKS.contains(u) && *u != unit(\',\')) &&', 'rules'),
    ('space-only-after-spaces', DICT + 'phrases.rs', 'if kept.last().is_some_and(|&u| !is_whitespace(u)) {',
     "if kept.last().is_some_and(|&u| u != unit(' ')) {", 'rules'),
    # Spoken commands: what ends one, spaces inside one, and where a sentence ends.
    ('comma-ends-no-command', DICT + 'spoken.rs', "let marks = [unit('.'), unit('!'), unit('?'), unit(',')];",
     "let marks = [unit('.'), unit('!'), unit('?'), unit(';')];", 'rules'),
    ('spaces-not-squeezed', DICT + 'spoken.rs', '} else if squeezed.last().is_none_or(|&last| !is_regex_space(last)) {',
     '} else if squeezed.last().is_some() {', 'rules'),
    ('exclamation-splits-nothing', DICT + 'spoken.rs', "[unit('.'), unit('!'), unit('?')].contains(&text[at - 1])",
     "[unit('.'), unit('?')].contains(&text[at - 1])", 'rules'),
    # Scratch that: only right after the phrase.
    ('reverts-past-the-phrase', DICT + 'undo.rs', 'cursor == start.wrapping_add(length)',
     'cursor >= start.wrapping_add(length)', 'rules'),
    ('replacing-off-by-one', GLUE, 'let replacing = u16::try_from(typing.replacing)',
     'let replacing = u16::try_from(typing.replacing + 1)', 'rules'),
    # The mic key: a hold's length, how it is measured, and what a press starts.
    ('hold-exclusive', DICT + 'mic_key.rs', 'now.wrapping_sub(press.at) >= HOLD_MS', 'now.wrapping_sub(press.at) > HOLD_MS',
     'mic-key'),
    ('hold-saturating', DICT + 'mic_key.rs', 'now.wrapping_sub(press.at) >= HOLD_MS',
     'now.saturating_sub(press.at) >= HOLD_MS', 'mic-key'),
    ('finishing-starts', DICT + 'mic_key.rs', 'let started = status == Status::Idle;',
     'let started = status != Status::Listening;', 'mic-key'),
    ('hold-301', DICT + 'mic_key.rs', 'pub const HOLD_MS: i64 = 300;', 'pub const HOLD_MS: i64 = 301;', 'mic-key'),
    ('end-hold-reads-as-finish', GLUE, '        Action::EndHold => 3,\n', '        Action::EndHold => 2,\n', 'mic-key'),
]
