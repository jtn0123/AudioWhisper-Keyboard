//! What Android tells the graph: the facts of one computation, and the answers of the platform
//! questions only it can answer (locale casing and ICU's non-spacing-mark predicate).
//!
//! A computation asks a question it has no answer for yet, carries on, and hands the question
//! back; the caller answers and computes again. A graph asks each question once.

use std::collections::HashMap;

use popup::order::{HintConfig, HintMode};

/// The facts Android reports for one computation, which `ComputingEvaluatorImpl` read.
#[derive(Clone, Debug, PartialEq)]
pub struct Facts {
    /// The keyboard state's raw value, with the evaluator's shift reset applied.
    pub state: u64,
    /// The editor's `inputAttributes.raw` and `imeOptions.raw`.
    pub input: i32,
    pub options: i32,
    /// The utility key setting, its action's ordinal, and whether the language switch shows.
    pub utility: bool,
    pub utility_action: i32,
    pub language_switch: bool,
    /// Whether the editor is a rich one, the device or keyguard is locked, and the primary clip
    /// can be pasted.
    pub rich_editor: bool,
    pub locked: bool,
    pub pastable: bool,
    /// The incognito setting's ordinal, and whether the editor asks for no learning.
    pub incognito: i32,
    pub no_learning: bool,
    /// How many subtypes there are, and whether dictation may type into the field.
    pub subtypes: i32,
    pub dictation: bool,
    /// The subtype's language name, the space key's label, and `Subtype.DEFAULT`'s, which the
    /// smartbar's evaluator names.
    pub language: Vec<u16>,
    pub default_language: Vec<u16>,
    /// The label each resource token (a key code, as `input::visual::label` names it) shows.
    pub resources: Vec<(i32, Vec<u16>)>,
    /// The subtype's currency set: each slot's code and label.
    pub slots: Vec<(i32, Vec<u16>)>,
    /// The hint settings a key's hint label is chosen with.
    pub hints: HintConfig,
}

impl Default for Facts {
    fn default() -> Self {
        Facts {
            state: 0,
            input: 0,
            options: 0,
            utility: false,
            utility_action: 0,
            language_switch: false,
            rich_editor: false,
            locked: false,
            pastable: false,
            incognito: 0,
            no_learning: false,
            subtypes: 0,
            dictation: false,
            language: Vec::new(),
            default_language: Vec::new(),
            resources: Vec::new(),
            slots: Vec::new(),
            hints: HintConfig {
                symbol: HintMode::Disabled,
                number: HintMode::Disabled,
                merge_hint_popups: false,
            },
        }
    }
}

/// What a question asks of a code point or a text.
pub mod ask {
    /// A code point's first code point, lowercased in the locale.
    pub const LOWER_CODE: i32 = 0;
    /// A code point's first code point, uppercased in the locale.
    pub const UPPER_CODE: i32 = 1;
    /// A text, lowercased in the locale.
    pub const LOWER_TEXT: i32 = 2;
    /// A text, uppercased in the locale.
    pub const UPPER_TEXT: i32 = 3;
    /// Whether a code point is a non-spacing mark (1) or not (0); the locale is ignored.
    pub const MARK: i32 = 4;
}

/// U+0300, the first code point of general category Mn in every Unicode version.
const FIRST_MARK: i32 = 0x300;

/// Which locale a question is in.
pub mod locale {
    /// The subtype's primary locale.
    pub const SUBTYPE: i32 = 0;
    /// `Subtype.DEFAULT`'s, which the default evaluator cases with.
    pub const DEFAULT: i32 = 1;
}

/// A question for Android: what `ask` gives for `value` (a code point in decimal, or a text) in
/// `locale`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Question {
    pub ask: i32,
    pub locale: i32,
    pub value: Vec<u16>,
}

/// The answers so far, and the questions still open.
#[derive(Clone, Debug, Default)]
pub struct Platform {
    codes: HashMap<(i32, i32, i32), i32>,
    texts: HashMap<(i32, i32), HashMap<Vec<u16>, Vec<u16>>>,
    pending: Vec<Question>,
    /// How many lookups found no answer yet, each counted, asked before or not.
    misses: usize,
}

impl Platform {
    /// `code` cased by `ask` in `locale`; `code` itself until it is answered.
    pub fn code(&mut self, ask: i32, locale: i32, code: i32) -> i32 {
        if let Some(&answer) = self.codes.get(&(ask, locale, code)) {
            return answer;
        }
        self.open(ask, locale, code.to_string().encode_utf16().collect());
        code
    }

    /// Whether `code` is a non-spacing mark; false until it is answered. No code point below
    /// U+0300, the first, is one, so those aren't asked.
    pub fn mark(&mut self, code: i32) -> bool {
        if code < FIRST_MARK {
            return false;
        }
        if let Some(&answer) = self.codes.get(&(ask::MARK, locale::SUBTYPE, code)) {
            return answer != 0;
        }
        self.open(
            ask::MARK,
            locale::SUBTYPE,
            code.to_string().encode_utf16().collect(),
        );
        false
    }

    /// `text` cased by `ask` in `locale`; `text` itself until it is answered.
    pub fn text(&mut self, ask: i32, locale: i32, text: &[u16]) -> Vec<u16> {
        if let Some(answer) = self.texts.get(&(ask, locale)).and_then(|map| map.get(text)) {
            return answer.clone();
        }
        self.open(ask, locale, text.to_vec());
        text.to_vec()
    }

    fn open(&mut self, ask: i32, locale: i32, value: Vec<u16>) {
        self.misses += 1;
        let question = Question { ask, locale, value };
        if !self.pending.contains(&question) {
            self.pending.push(question);
        }
    }

    /// How many lookups have found no answer: one that grows across a step means the step used a
    /// stand-in, even for a question already open.
    pub fn misses(&self) -> usize {
        self.misses
    }

    /// The questions still open.
    pub fn pending(&self) -> &[Question] {
        &self.pending
    }

    /// Takes `answers` to the open questions, in order. Fails, keeping every question open, if
    /// there isn't one answer for each or a code's answer isn't a number.
    pub fn answer(&mut self, answers: &[Vec<u16>]) -> Result<(), &'static str> {
        if answers.len() != self.pending.len() {
            return Err("one answer is needed for each question");
        }
        let mut codes = Vec::new();
        for (question, answer) in self.pending.iter().zip(answers) {
            if matches!(question.ask, ask::LOWER_CODE | ask::UPPER_CODE | ask::MARK) {
                let code = String::from_utf16(&question.value)
                    .ok()
                    .and_then(|it| it.parse().ok());
                let answer = String::from_utf16(answer)
                    .ok()
                    .and_then(|it| it.parse().ok());
                codes.push((
                    question,
                    code.zip(answer).ok_or("a code's answer is a number")?,
                ));
            }
        }
        for (question, (code, answer)) in codes {
            self.codes
                .insert((question.ask, question.locale, code), answer);
        }
        for (question, answer) in std::mem::take(&mut self.pending).into_iter().zip(answers) {
            if matches!(question.ask, ask::LOWER_TEXT | ask::UPPER_TEXT) {
                let map = self
                    .texts
                    .entry((question.ask, question.locale))
                    .or_default();
                map.insert(question.value, answer.clone());
            }
        }
        Ok(())
    }

    /// Drops the open questions, whose answers a computation that asked them no longer needs.
    pub fn forget_pending(&mut self) {
        self.pending.clear();
    }
}
