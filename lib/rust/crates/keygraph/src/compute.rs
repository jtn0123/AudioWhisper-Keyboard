//! Computing a key for the keyboard's state: `AbstractKeyData.compute`, and what
//! `ComputingEvaluatorImpl` decides of the result (visible, enabled, label, icon).

use input::protocol::*;
use keys::{codes, models};

use crate::data::Data;
use crate::facts::{Facts, Platform, ask};
use crate::node::{Leaf, Node};

/// Who computes: the live evaluator, or FlorisBoard's `DefaultComputingEvaluator`, which reads
/// a zero state, cases for `Subtype.DEFAULT` and resolves no currency slot.
#[derive(Clone, Copy, Debug)]
pub struct Eval<'a> {
    pub state: u64,
    /// The currency slots, or `None` for the default evaluator.
    pub slots: Option<&'a [(i32, Vec<u16>)]>,
    /// The `facts::locale` letters are cased in.
    pub locale: i32,
}

/// The icons Android draws, by the token `input::visual::icon` picks; any other token draws none.
pub const DRAWN: &[i32] = &[
    codes::ARROW_LEFT,
    codes::ARROW_RIGHT,
    codes::ARROW_UP,
    codes::ARROW_DOWN,
    codes::CLIPBOARD_COPY,
    codes::CLIPBOARD_CUT,
    codes::CLIPBOARD_PASTE,
    codes::CLIPBOARD_SELECT_ALL,
    codes::CLIPBOARD_CLEAR_PRIMARY_CLIP,
    codes::COMPACT_LAYOUT_TO_LEFT,
    codes::COMPACT_LAYOUT_TO_RIGHT,
    codes::TOGGLE_COMPACT_LAYOUT,
    codes::VOICE_INPUT,
    codes::READ_ALOUD,
    codes::IME_HIDE_UI,
    codes::DELETE,
    ICON_RETURN,
    ICON_DONE,
    ICON_ARROW,
    ICON_SEARCH,
    ICON_SEND,
    codes::FORWARD_DELETE,
    codes::IME_UI_MODE_MEDIA,
    codes::IME_UI_MODE_CLIPBOARD,
    codes::LANGUAGE_SWITCH,
    codes::SETTINGS,
    ICON_CAPS,
    codes::SHIFT,
    codes::SPACE,
    codes::UNDO,
    codes::REDO,
    codes::TOGGLE_ACTIONS_OVERFLOW,
    codes::TOGGLE_INCOGNITO_MODE,
    ICON_INCOGNITO_OFF,
    codes::TOGGLE_AUTOCORRECT,
    codes::CHAR_WIDTH_FULL,
    codes::CHAR_WIDTH_HALF,
    codes::NOOP,
];

/// What `node` computes to, or `None` for nothing.
pub fn node(node: &Node, eval: Eval, platform: &mut Platform) -> Option<Data> {
    match node {
        Node::Text(leaf) => slot(leaf_data(leaf, None), eval),
        Node::Auto(leaf, _) if eval.slots.is_some() && models::is_currency(leaf.code) => {
            slot(leaf_data(leaf, None), eval)
        }
        Node::Auto(leaf, _) => Some(cased(leaf, eval, platform)),
        Node::Multi(leaf, id) => Some(leaf_data(leaf, Some(*id))),
        Node::Select(kind, children) => {
            let present = children
                .iter()
                .enumerate()
                .filter(|(_, child)| child.is_some())
                .fold(0u32, |bits, (at, _)| bits | (1 << at));
            let at = usize::try_from(models::select(*kind, eval.state, present)).ok()?;
            let child = children.get(at)?.as_ref()?;
            self::node(child, eval, platform)
        }
    }
}

/// A computed key computed again, as a `KeyData`'s own `compute`: a multi-text key is itself, a
/// text key its currency slot if it names one.
pub fn again(data: &Data, eval: Eval) -> Option<Data> {
    if data.multi.is_some() {
        return Some(data.clone());
    }
    slot(data.clone(), eval)
}

fn leaf_data(leaf: &Leaf, multi: Option<usize>) -> Data {
    Data {
        kind: leaf.kind,
        code: leaf.code,
        label: leaf.label.clone(),
        group: leaf.group,
        popup: leaf.popup,
        multi,
    }
}

/// `data`, or the currency slot it names when the evaluator resolves slots: `None` if the set
/// has no such slot.
fn slot(data: Data, eval: Eval) -> Option<Data> {
    let Some(slots) = eval.slots.filter(|_| models::is_currency(data.code)) else {
        return Some(data);
    };
    let at = usize::try_from(models::currency_slot(data.code)).ok()?;
    let (code, label) = slots.get(at)?;
    Some(Data {
        code: *code,
        label: label.clone(),
        multi: None,
        ..data
    })
}

/// An `auto_text_key`'s lower or upper case, by the state's shift.
fn cased(leaf: &Leaf, eval: Eval, platform: &mut Platform) -> Data {
    let upper = models::select(0, eval.state, 3) != 0;
    let (code, text) = if upper {
        (ask::UPPER_CODE, ask::UPPER_TEXT)
    } else {
        (ask::LOWER_CODE, ask::LOWER_TEXT)
    };
    Data {
        kind: leaf.kind,
        code: platform.code(code, eval.locale, leaf.code),
        label: platform.text(text, eval.locale, &leaf.label),
        group: leaf.group,
        popup: leaf.popup,
        multi: None,
    }
}

/// Whether a key with `code` shows (`evaluateVisible`).
pub fn visible(facts: &Facts, code: i32) -> bool {
    models::visible(
        code,
        facts.utility,
        facts.utility_action,
        facts.language_switch,
    )
}

/// Whether a key with `code` is enabled (`evaluateEnabled`), from the facts it reads.
pub fn enabled(facts: &Facts, code: i32) -> bool {
    let bit = i32::from;
    let selecting = models::read(facts.state, 5) == Some(1);
    let (a, b) = match code {
        codes::CLIPBOARD_COPY | codes::CLIPBOARD_CUT => (bit(selecting), bit(facts.rich_editor)),
        codes::CLIPBOARD_PASTE => (bit(facts.locked), bit(!facts.locked && facts.pastable)),
        codes::CLIPBOARD_CLEAR_PRIMARY_CLIP => (bit(facts.pastable), 0),
        codes::CLIPBOARD_SELECT_ALL => (bit(facts.rich_editor), 0),
        codes::TOGGLE_INCOGNITO_MODE => (facts.incognito, bit(facts.no_learning)),
        codes::LANGUAGE_SWITCH => (facts.subtypes, 0),
        codes::VOICE_INPUT => (bit(facts.dictation), 0),
        _ => (0, 0),
    };
    models::enabled(code, a, b)
}

/// The label a key shows on a keyboard of `mode` (`computeLabel`).
pub fn label(data: &Data, mode: i32, facts: &Facts, platform: &mut Platform) -> Option<Vec<u16>> {
    match input::visual::label(data.kind, data.code, mode) {
        0 => None,
        1 => Some(text(data, platform)),
        2 => Some(facts.language.clone()),
        token => facts
            .resources
            .iter()
            .find(|(resource, _)| *resource == token)
            .map(|(_, label)| label.clone()),
    }
}

/// The key's text for display (`asString(isForDisplay = true)`).
pub fn text(data: &Data, platform: &mut Platform) -> Vec<u16> {
    if data.multi.is_some() {
        return data.label.clone();
    }
    models::text(data.code, &data.label, true, platform.mark(data.code))
}

/// The icon a key draws on a keyboard of `mode` (`computeImageVector`), 0 for none.
pub fn icon(data: &Data, mode: i32, facts: &Facts) -> i32 {
    let token = input::visual::icon(data.code, facts.state, facts.input, facts.options, mode);
    if DRAWN.contains(&token) { token } else { 0 }
}
