//! Whether to type a space of the keyboard's own around what is being typed, as
//! `EditorInstance`'s `shouldInsertAutoSpaceBefore`, `shouldInsertAutoSpaceAfter`,
//! `PhantomSpaceState.determine` and `commitChar` decided, and `FlorisLocale.supportsAutoSpace`.
//!
//! Each rule checks what costs nothing first (the setting, the editor, the flags) and only then
//! the text before the cursor, which Kotlin must cut by ICU's grapheme breaks. So the text comes
//! in an [`Editor`] that may be missing: a rule that gets as far as needing it answers
//! `Err(needs)`, and Kotlin looks the text up and asks again.

use crate::rule::{Rule, lists};
use crate::{is_ascii, state};

/// `KeyVariation.NORMAL`'s value: auto-spacing is only for ordinary text, not addresses or
/// passwords.
pub const NORMAL_VARIATION: i32 = 2;

/// Which text a rule needs before it can answer, as bits.
pub type Needs = u8;
/// The last character before the cursor (`getTextBeforeCursor(1)`).
pub const NEEDS_BEFORE_1: Needs = 1;
/// The last three characters before the cursor (`getTextBeforeCursor(3)`) and the current word.
pub const NEEDS_BEFORE_3: Needs = 2;

const SPACE: u16 = b' ' as u16;

/// The languages whose words aren't spaced apart: no phantom space is typed in them. "jp" is
/// upstream's, for Japanese, whose code is "ja", so Japanese still gets phantom spaces; it is kept,
/// as the port changes nothing.
const UNSPACED_LANGUAGES: [&str; 4] = ["zh", "ko", "jp", "th"];

/// What is known before any text is looked up.
#[derive(Clone, Copy, Debug)]
pub struct Typing {
    /// The setting to space after punctuation (`prefs.correction.autoSpacePunctuation`).
    pub auto_space_punctuation: bool,
    /// The editor takes key events, not text (`activeInfo.isRawInputEditor`).
    pub raw_editor: bool,
    /// The keyboard's `KeyVariation.value`.
    pub key_variation: i32,
    /// The editor's selection is valid (`selection.isValid`).
    pub selection_valid: bool,
    /// Where it starts.
    pub selection_start: i32,
}

/// What Kotlin looked up: the rule and language always, and the texts a rule asked for.
#[derive(Clone, Copy, Debug, Default)]
pub struct Editor<'a> {
    pub rule: Rule<'a>,
    /// The subtype's primary language (`FlorisLocale.language`).
    pub language: &'a [u16],
    /// `getTextBeforeCursor(1)`, if [`NEEDS_BEFORE_1`] was asked for.
    pub before_1: Option<&'a [u16]>,
    /// `getTextBeforeCursor(3)`, if [`NEEDS_BEFORE_3`] was asked for.
    pub before_3: Option<&'a [u16]>,
    /// `currentWordText`, if [`NEEDS_BEFORE_3`] was asked for.
    pub current_word: Option<&'a [u16]>,
}

/// What to do around a typed character, and the flags after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Commit {
    /// Take back the auto-space typed before (the character is punctuation that goes against
    /// what came before it).
    pub delete_previous_space: bool,
    /// Type a space before the character: an auto-space or a phantom space.
    pub space_before: bool,
    /// Type an auto-space after it.
    pub space_after: bool,
    /// The auto-space flag after it.
    pub auto_space: i32,
    /// The phantom-space flag after it.
    pub phantom_space: i32,
}

/// Whether to type a space after a word in `language` (`FlorisLocale.supportsAutoSpace`).
pub fn supports_auto_space(language: &[u16]) -> bool {
    !UNSPACED_LANGUAGES
        .iter()
        .any(|&code| is_ascii(language, code))
}

fn auto_spaces(typing: &Typing) -> bool {
    typing.auto_space_punctuation && !typing.raw_editor && typing.key_variation == NORMAL_VARIATION
}

fn ends_in_other_than_whitespace(text: &[u16]) -> bool {
    text.last().is_some_and(|&last| !chars::is_whitespace(last))
}

/// The editor, with `getTextBeforeCursor(1)`.
fn before_1<'a>(editor: Option<&Editor<'a>>) -> Result<(Editor<'a>, &'a [u16]), Needs> {
    editor
        .and_then(|editor| editor.before_1.map(|before| (*editor, before)))
        .ok_or(NEEDS_BEFORE_1)
}

/// Whether typing `text` types an auto-space before it (`shouldInsertAutoSpaceBefore`).
pub fn auto_space_before(
    typing: &Typing,
    text: &[u16],
    editor: Option<&Editor>,
) -> Result<bool, Needs> {
    let Some(&first) = text.first() else {
        return Ok(false);
    };
    if !auto_spaces(typing) {
        return Ok(false);
    }
    let (editor, before) = before_1(editor)?;
    Ok(ends_in_other_than_whitespace(before) && lists(editor.rule.following_auto_space, first))
}

/// Whether typing `text` types an auto-space after it (`shouldInsertAutoSpaceAfter`), with the
/// auto-space flag `auto_space`.
pub fn auto_space_after(
    typing: &Typing,
    text: &[u16],
    auto_space: i32,
    editor: Option<&Editor>,
) -> Result<bool, Needs> {
    let Some(&first) = text.first() else {
        return Ok(false);
    };
    if !auto_spaces(typing) {
        return Ok(false);
    }
    let (editor, before, word) = editor
        .and_then(|editor| Some((*editor, editor.before_3?, editor.current_word?)))
        .ok_or(NEEDS_BEFORE_3)?;
    // The auto-space typed last doesn't count as what the punctuation follows.
    let before = match before.split_last() {
        Some((&SPACE, rest)) if state::is_active(auto_space) => rest,
        _ => before,
    };
    Ok(ends_in_other_than_whitespace(before)
        && word.iter().all(|&unit| !chars::is_digit(unit))
        && lists(editor.rule.preceding_auto_space, first))
}

/// Whether to type the phantom space before `text` (`PhantomSpaceState.determine`): if one is
/// owed (`phantom_space`) or `force_active`, after a letter, a digit or a symbol a phantom space
/// may follow, and before a letter, a digit or a symbol it may come before.
pub fn phantom_space_before(
    typing: &Typing,
    text: &[u16],
    phantom_space: i32,
    force_active: bool,
    editor: Option<&Editor>,
) -> Result<bool, Needs> {
    let Some(&first) = text.first() else {
        return Ok(false);
    };
    if !(state::is_active(phantom_space) || force_active)
        || !typing.selection_valid
        || typing.selection_start <= 0
    {
        return Ok(false);
    }
    let (editor, before) = before_1(editor)?;
    if !supports_auto_space(editor.language) {
        return Ok(false);
    }
    let rule = editor.rule;
    Ok(before.last().is_some_and(|&last| {
        lists(rule.preceding_phantom_space, last) || chars::is_letter_or_digit(last)
    }) && (lists(rule.following_phantom_space, first) || chars::is_letter_or_digit(first)))
}

/// What typing the character `text` does with spaces (`EditorInstance.commitChar`), with the
/// flags `auto_space` and `phantom_space` as they were.
pub fn commit_char(
    typing: &Typing,
    text: &[u16],
    auto_space: i32,
    phantom_space: i32,
    editor: Option<&Editor>,
) -> Result<Commit, Needs> {
    let before = auto_space_before(typing, text, editor);
    let after = auto_space_after(typing, text, auto_space, editor);
    let phantom = phantom_space_before(typing, text, phantom_space, false, editor);
    let (Ok(before), Ok(after), Ok(phantom)) = (before, after, phantom) else {
        let needs = |answer: Result<bool, Needs>| answer.err().unwrap_or(0);
        return Err(needs(before) | needs(after) | needs(phantom));
    };
    Ok(Commit {
        delete_previous_space: after && state::is_active(auto_space),
        space_before: before || phantom,
        space_after: after,
        auto_space: if after {
            state::auto_space_active(true)
        } else {
            state::INACTIVE
        },
        phantom_space: state::INACTIVE,
    })
}

#[cfg(test)]
mod tests;
