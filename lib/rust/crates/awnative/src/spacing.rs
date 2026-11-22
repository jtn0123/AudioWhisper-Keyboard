//! The spaces the keyboard types by itself (the `spacing` crate), for `SpacingNative`, whose
//! generated entry points call these or the crate directly.
//!
//! A rule first runs on what is known before any text is looked up. If it gets far enough to need
//! more, Kotlin's `SpacingRules.Editor` is asked through JNI for that and no more, and the rule
//! runs again.
//! A commit's answer is one `int` of bits ([`answer`]).

use std::ffi::CStr;

use jni_sys::{jobject, jvalue};
use spacing::rule::{Rule, fallback};
use spacing::spaces::{self, Commit, Editor, NEEDS_BEFORE_1, NEEDS_BEFORE_3, Needs, Typing};

use crate::boundary::args::{SpacingNativeCommitChar, SpacingNativePhantomSpaceBefore};
use crate::jni::{Env, Result, illegal};

// The bits of an answer; the flags after it sit above them.
const DELETE_PREVIOUS_SPACE: i32 = 1;
const SPACE_BEFORE: i32 = 2;
const SPACE_AFTER: i32 = 4;
const AUTO_SPACE_SHIFT: u32 = 8;
const PHANTOM_SPACE_SHIFT: u32 = 16;

/// A method of `SpacingRules.Editor` or `SpacingRules.Rule` that returns a string: its name and
/// JNI signature.
type Getter = (&'static CStr, &'static CStr);

const RULE: Getter = (
    c"getRule",
    c"()Lcom/jtn0123/audiowhisper/rust/SpacingRules$Rule;",
);
const SYMBOLS: [Getter; 4] = [
    (c"getPrecedingAutoSpace", c"()Ljava/lang/String;"),
    (c"getFollowingAutoSpace", c"()Ljava/lang/String;"),
    (c"getPrecedingPhantomSpace", c"()Ljava/lang/String;"),
    (c"getFollowingPhantomSpace", c"()Ljava/lang/String;"),
];
const LANGUAGE: Getter = (c"getLanguage", c"()Ljava/lang/String;");
const TEXT_BEFORE_CURSOR: Getter = (c"textBeforeCursor", c"(I)Ljava/lang/String;");
const CURRENT_WORD: Getter = (c"getCurrentWordText", c"()Ljava/lang/String;");

/// What `object`'s `getter` gives with `arguments`.
fn string(
    env: &Env,
    object: jobject,
    what: &'static str,
    getter: Getter,
    arguments: &[jvalue],
) -> Result<Vec<u16>> {
    let method = env.method(object, what, getter)?;
    let string = env.call_object(method, arguments)?;
    let text = env.read_string(string, what);
    if !string.is_null() {
        env.delete_local(string);
    }
    text
}

/// The four symbol strings of `rule`, a `SpacingRules.Rule`.
fn symbols(env: &Env, rule: jobject) -> Result<[Vec<u16>; 4]> {
    let mut symbols: [Vec<u16>; 4] = Default::default();
    for (symbol, getter) in symbols.iter_mut().zip(SYMBOLS) {
        *symbol = string(env, rule, "rule", getter, &[])?;
    }
    Ok(symbols)
}

/// What the editor was asked for.
struct LookedUp {
    symbols: [Vec<u16>; 4],
    language: Vec<u16>,
    before_1: Option<Vec<u16>>,
    before_3: Option<Vec<u16>>,
    current_word: Option<Vec<u16>>,
}

impl LookedUp {
    /// Asks `editor` for what `needs` says a rule needs, in the Kotlin's order: the rule, the
    /// texts before the cursor, the language, then the current word.
    fn ask(env: &Env, editor: jobject, needs: Needs) -> Result<LookedUp> {
        let method = env.method(editor, "editor", RULE)?;
        let rule = env.call_object(method, &[])?;
        let symbols = symbols(env, rule);
        if !rule.is_null() {
            env.delete_local(rule);
        }
        let symbols = symbols?;
        let before = |n: i32| {
            string(
                env,
                editor,
                "editor",
                TEXT_BEFORE_CURSOR,
                &[jvalue { i: n }],
            )
        };
        let before_1 = (needs & NEEDS_BEFORE_1 != 0)
            .then(|| before(1))
            .transpose()?;
        let before_3 = (needs & NEEDS_BEFORE_3 != 0)
            .then(|| before(3))
            .transpose()?;
        let language = string(env, editor, "editor", LANGUAGE, &[])?;
        let current_word = match before_3 {
            Some(_) => Some(string(env, editor, "editor", CURRENT_WORD, &[])?),
            None => None,
        };
        Ok(LookedUp {
            symbols,
            language,
            before_1,
            before_3,
            current_word,
        })
    }

    fn editor(&self) -> Editor<'_> {
        let [
            preceding_auto,
            following_auto,
            preceding_phantom,
            following_phantom,
        ] = &self.symbols;
        Editor {
            rule: Rule {
                preceding_auto_space: preceding_auto,
                following_auto_space: following_auto,
                preceding_phantom_space: preceding_phantom,
                following_phantom_space: following_phantom,
            },
            language: &self.language,
            before_1: self.before_1.as_deref(),
            before_3: self.before_3.as_deref(),
            current_word: self.current_word.as_deref(),
        }
    }
}

/// `rule`'s answer, asking `editor` for what it needs if it gets that far.
fn ask<T>(
    env: &Env,
    editor: jobject,
    rule: impl Fn(Option<&Editor>) -> std::result::Result<T, Needs>,
) -> Result<T> {
    let needs = match rule(None) {
        Ok(answer) => return Ok(answer),
        Err(needs) => needs,
    };
    let looked_up = LookedUp::ask(env, editor, needs)?;
    rule(Some(&looked_up.editor())).map_err(|still| illegal(format!("still needs {still}")))
}

/// A state as the 8 bits an answer has room for.
fn state_bits(state: i32) -> Result<i32> {
    u8::try_from(state)
        .map(i32::from)
        .map_err(|_| illegal("a state past 8 bits"))
}

/// An answer's bits: whether to delete the space before, type one before and type one after, then
/// the auto-space flag after it from bit 8 and the phantom-space flag from bit 16.
fn answer(commit: &Commit) -> Result<i32> {
    let bit = |set: bool, bit: i32| if set { bit } else { 0 };
    Ok(bit(commit.delete_previous_space, DELETE_PREVIOUS_SPACE)
        | bit(commit.space_before, SPACE_BEFORE)
        | bit(commit.space_after, SPACE_AFTER)
        | state_bits(commit.auto_space)? << AUTO_SPACE_SHIFT
        | state_bits(commit.phantom_space)? << PHANTOM_SPACE_SHIFT)
}

/// Typing the character `text` (`spaces::commit_char`): the spaces around it and the flags after
/// it.
pub fn commit_char(env: &Env, args: SpacingNativeCommitChar) -> Result<i32> {
    let typing = Typing {
        auto_space_punctuation: args.auto_space_punctuation,
        raw_editor: args.raw_editor,
        key_variation: args.key_variation,
        selection_valid: args.selection_valid,
        selection_start: args.selection_start,
        input_type: args.input_type,
    };
    let commit = ask(env, args.editor, |editor| {
        spaces::commit_char(
            &typing,
            args.text,
            args.auto_space,
            args.phantom_space,
            editor,
        )
    })?;
    answer(&commit)
}

/// Whether to type the phantom space before `text` (`spaces::phantom_space_before`).
pub fn phantom_space_before(env: &Env, args: SpacingNativePhantomSpaceBefore) -> Result<bool> {
    let typing = Typing {
        auto_space_punctuation: args.auto_space_punctuation,
        raw_editor: args.raw_editor,
        key_variation: args.key_variation,
        selection_valid: args.selection_valid,
        selection_start: args.selection_start,
        input_type: args.input_type,
    };
    ask(env, args.editor, |editor| {
        spaces::phantom_space_before(
            &typing,
            args.text,
            args.phantom_space,
            args.force_active,
            editor,
        )
    })
}

/// `PunctuationRule.Fallback`'s id, label and symbols, in its constructor's order.
pub fn fallback_rule() -> Vec<Vec<u16>> {
    [
        fallback::ID,
        fallback::LABEL,
        fallback::SYMBOLS_PRECEDING_AUTO_SPACE,
        fallback::SYMBOLS_FOLLOWING_AUTO_SPACE,
        fallback::SYMBOLS_PRECEDING_PHANTOM_SPACE,
        fallback::SYMBOLS_FOLLOWING_PHANTOM_SPACE,
        fallback::SYMBOLS_TERMINATING_SENTENCE,
    ]
    .iter()
    .map(|text| text.encode_utf16().collect())
    .collect()
}

/// Neither flag set (`state::INACTIVE`).
pub fn inactive() -> i32 {
    spacing::state::INACTIVE
}
