//! JNI entry points for the spaces the keyboard types by itself (the `spacing` crate), called by
//! `SpacingNative` in lib/rust's Kotlin.
//!
//! A rule's inputs cross as the text being typed, an `int[]` of what is known before any text is
//! looked up ([`typing`]'s layout), and, once a rule has asked for it, what was looked up: one
//! string holding the rule's symbols, the language and the texts, cut by an `int[]` of their
//! lengths (-1 for a text not looked up). The answer is one `int` of bits ([`answer`]).

use std::ptr;

use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jclass, jint, jintArray, jobjectArray, jstring};
use spacing::rule::{Rule, fallback};
use spacing::spaces::{self, Commit, Editor, Needs, Typing};
use spacing::state;

use crate::jni::{self, Env, Result};

// The bits of `typing[FLAGS]`.
const AUTO_SPACE_PUNCTUATION: i32 = 1;
const RAW_EDITOR: i32 = 2;
const SELECTION_VALID: i32 = 4;
const FORCE_ACTIVE: i32 = 8;

// The bits of an answer; the needs and the flags after it sit above them.
const DELETE_PREVIOUS_SPACE: i32 = 1;
const SPACE_BEFORE: i32 = 2;
const SPACE_AFTER: i32 = 4;
const NEEDS_SHIFT: u32 = 3;
const AUTO_SPACE_SHIFT: u32 = 8;
const PHANTOM_SPACE_SHIFT: u32 = 16;

/// The pieces of the looked-up string, in order.
const PIECES: usize = 8;

/// What is known before any text is looked up, and the flags, from `typing`: `[flags,
/// keyVariation, selectionStart, autoSpace, phantomSpace, inputType]`.
struct Known {
    typing: Typing,
    force_active: bool,
    auto_space: i32,
    phantom_space: i32,
}

fn typing(env: &Env, typing: jintArray) -> Result<Known> {
    let values = env.read::<i32>(typing, "typing")?;
    let &[
        flags,
        key_variation,
        selection_start,
        auto_space,
        phantom_space,
        input_type,
    ] = values.as_slice()
    else {
        return Err(jni::illegal("typing holds six ints"));
    };
    Ok(Known {
        typing: Typing {
            auto_space_punctuation: flags & AUTO_SPACE_PUNCTUATION != 0,
            raw_editor: flags & RAW_EDITOR != 0,
            key_variation,
            selection_valid: flags & SELECTION_VALID != 0,
            selection_start,
            input_type,
        },
        force_active: flags & FORCE_ACTIVE != 0,
        auto_space,
        phantom_space,
    })
}

/// What was looked up, cut into its pieces: `[precedingAutoSpace, followingAutoSpace,
/// precedingPhantomSpace, followingPhantomSpace, language, before1, before3, currentWord]`.
fn pieces<'a>(text: &'a [u16], lengths: &[i32]) -> Result<[Option<&'a [u16]>; PIECES]> {
    if lengths.len() != PIECES {
        return Err(jni::illegal("eight lengths"));
    }
    let cut = jni::cut(text, &lengths.iter().map(|&n| n.max(0)).collect::<Vec<_>>())?;
    let mut pieces = [None; PIECES];
    for (piece, (&units, &length)) in pieces.iter_mut().zip(cut.iter().zip(lengths)) {
        if length >= 0 {
            *piece = Some(units);
        } else if length != -1 {
            return Err(jni::illegal("a negative length"));
        }
    }
    Ok(pieces)
}

fn editor<'a>(pieces: &[Option<&'a [u16]>; PIECES]) -> Result<Editor<'a>> {
    let symbols = |i: usize| pieces[i].ok_or_else(|| jni::illegal("the rule's symbols"));
    Ok(Editor {
        rule: Rule {
            preceding_auto_space: symbols(0)?,
            following_auto_space: symbols(1)?,
            preceding_phantom_space: symbols(2)?,
            following_phantom_space: symbols(3)?,
        },
        language: pieces[4].ok_or_else(|| jni::illegal("the language"))?,
        before_1: pieces[5],
        before_3: pieces[6],
        current_word: pieces[7],
    })
}

/// A state as the 8 bits an answer has room for.
fn state_bits(state: i32) -> Result<i32> {
    u8::try_from(state)
        .map(i32::from)
        .map_err(|_| jni::illegal("a state past 8 bits"))
}

fn needs(needs: Needs) -> i32 {
    i32::from(needs) << NEEDS_SHIFT
}

/// An answer's bits.
fn answer(commit: &Commit) -> Result<i32> {
    let bit = |set: bool, bit: i32| if set { bit } else { 0 };
    Ok(bit(commit.delete_previous_space, DELETE_PREVIOUS_SPACE)
        | bit(commit.space_before, SPACE_BEFORE)
        | bit(commit.space_after, SPACE_AFTER)
        | state_bits(commit.auto_space)? << AUTO_SPACE_SHIFT
        | state_bits(commit.phantom_space)? << PHANTOM_SPACE_SHIFT)
}

/// Reads the arguments every rule takes and asks `rule` with them.
fn with_inputs(
    env: &Env,
    text: jstring,
    typing_values: jintArray,
    looked_up: jstring,
    lengths: jintArray,
    rule: impl FnOnce(&[u16], &Known, Option<&Editor>) -> Result<i32>,
) -> Result<i32> {
    let text = env.read_string(text, "text")?;
    let known = typing(env, typing_values)?;
    let Some(looked_up) = env.read_nullable_string(looked_up)? else {
        return rule(&text, &known, None);
    };
    let lengths = env.read::<i32>(lengths, "lengths")?;
    let pieces = pieces(&looked_up, &lengths)?;
    rule(&text, &known, Some(&editor(&pieces)?))
}

/// Typing the character `text` (`spaces::commit_char`): the spaces around it and the flags after
/// it, or what it needs looked up.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SpacingNative_commitChar(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
    typing: jintArray,
    looked_up: jstring,
    lengths: jintArray,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            with_inputs(
                env,
                text,
                typing,
                looked_up,
                lengths,
                |text, known, editor| match spaces::commit_char(
                    &known.typing,
                    text,
                    known.auto_space,
                    known.phantom_space,
                    editor,
                ) {
                    Ok(commit) => answer(&commit),
                    Err(needed) => Ok(needs(needed)),
                },
            )
        })
    }
}

/// Whether to type the phantom space before `text` (`spaces::phantom_space_before`), as
/// [`SPACE_BEFORE`], or what it needs looked up.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SpacingNative_phantomSpaceBefore(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
    typing: jintArray,
    looked_up: jstring,
    lengths: jintArray,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            with_inputs(
                env,
                text,
                typing,
                looked_up,
                lengths,
                |text, known, editor| match spaces::phantom_space_before(
                    &known.typing,
                    text,
                    known.phantom_space,
                    known.force_active,
                    editor,
                ) {
                    Ok(true) => Ok(SPACE_BEFORE),
                    Ok(false) => Ok(0),
                    Err(needed) => Ok(needs(needed)),
                },
            )
        })
    }
}

/// Whether a word in `language` is followed by a space (`spaces::supports_auto_space`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SpacingNative_supportsAutoSpace(
    env: *mut JNIEnv,
    _: jclass,
    language: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, JNI_FALSE, |env| {
            let language = env.read_string(language, "language")?;
            Ok(jni::boolean(spaces::supports_auto_space(&language)))
        })
    }
}

/// `PunctuationRule.Fallback`'s id, label and symbols, in its constructor's order.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SpacingNative_fallbackRule(
    env: *mut JNIEnv,
    _: jclass,
) -> jobjectArray {
    let rule: Vec<Vec<u16>> = [
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
    .collect();
    // SAFETY: the JVM passes its environment for this call.
    unsafe { jni::run(env, ptr::null_mut(), |env| env.make_strings(&rule)) }
}

/// Neither flag set (`state::INACTIVE`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SpacingNative_inactive(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    state::INACTIVE
}

/// An active auto-space (`state::auto_space_active`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SpacingNative_autoSpaceActive(
    _: *mut JNIEnv,
    _: jclass,
    stay_active_next_update: jboolean,
) -> jint {
    state::auto_space_active(stay_active_next_update != JNI_FALSE)
}

/// An active phantom space (`state::phantom_space_active`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SpacingNative_phantomSpaceActive(
    _: *mut JNIEnv,
    _: jclass,
    show_composing_region: jboolean,
    stay_active_next_update: jboolean,
) -> jint {
    state::phantom_space_active(
        show_composing_region != JNI_FALSE,
        stay_active_next_update != JNI_FALSE,
    )
}

/// Whether the space is owed (`state::is_active`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SpacingNative_isActive(
    _: *mut JNIEnv,
    _: jclass,
    state: jint,
) -> jboolean {
    jni::boolean(state::is_active(state))
}

/// Whether the word before a phantom space shows as composed (`state::shows_composing_region`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SpacingNative_showsComposingRegion(
    _: *mut JNIEnv,
    _: jclass,
    state: jint,
) -> jboolean {
    jni::boolean(state::shows_composing_region(state))
}

/// The state after a selection update (`state::after_update`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SpacingNative_afterUpdate(
    _: *mut JNIEnv,
    _: jclass,
    state: jint,
) -> jint {
    state::after_update(state)
}

/// Whether a phantom space keeps its suggestion through an update (`state::keeps_candidate`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SpacingNative_keepsCandidate(
    _: *mut JNIEnv,
    _: jclass,
    state: jint,
) -> jboolean {
    jni::boolean(state::keeps_candidate(state))
}
