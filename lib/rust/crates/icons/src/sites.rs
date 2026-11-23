//! Which icon each place draws, as the Kotlin that sent the paths chose them: `keyIconVector`
//! for a key's token, `SmartbarShown.icon` for an action, `ClipboardSuggestionCandidate.icon`
//! and the panels' fixed lists. Names are Compose's (`Filled.Search`,
//! `AutoMirrored.Outlined.Backspace`) or a drawable's (`drawable.ic_incognito`).

use keys::codes::*;
use wire::input::*;
use wire::smartbar::icon;

/// The icon a key's `token` draws: a key code, or an `ICON_` token `input::visual::icon` picks.
pub fn for_key(token: i32) -> Option<&'static str> {
    Some(match token {
        ARROW_LEFT => "AutoMirrored.Filled.KeyboardArrowLeft",
        ARROW_RIGHT => "AutoMirrored.Filled.KeyboardArrowRight",
        ARROW_UP | SHIFT => "Filled.KeyboardArrowUp",
        ARROW_DOWN => "Filled.KeyboardArrowDown",
        CLIPBOARD_COPY => "Filled.ContentCopy",
        CLIPBOARD_CUT => "Filled.ContentCut",
        CLIPBOARD_PASTE => "Filled.ContentPasteGo",
        CLIPBOARD_SELECT_ALL => "Filled.SelectAll",
        CLIPBOARD_CLEAR_PRIMARY_CLIP => "Filled.DeleteSweep",
        COMPACT_LAYOUT_TO_LEFT | COMPACT_LAYOUT_TO_RIGHT | TOGGLE_COMPACT_LAYOUT => {
            "drawable.ic_accessibility_one_handed"
        }
        VOICE_INPUT => "Filled.KeyboardVoice",
        READ_ALOUD => "AutoMirrored.Filled.VolumeUp",
        IME_HIDE_UI => "Filled.KeyboardHide",
        DELETE => "AutoMirrored.Outlined.Backspace",
        ICON_RETURN => "AutoMirrored.Filled.KeyboardReturn",
        ICON_DONE => "Filled.Done",
        ICON_ARROW => "AutoMirrored.Filled.ArrowRightAlt",
        ICON_SEARCH => "Filled.Search",
        ICON_SEND => "AutoMirrored.Filled.Send",
        FORWARD_DELETE => "AutoMirrored.Filled.ForwardDelete",
        IME_UI_MODE_MEDIA => "Filled.SentimentSatisfiedAlt",
        IME_UI_MODE_CLIPBOARD => "AutoMirrored.Outlined.Assignment",
        LANGUAGE_SWITCH => "Filled.Language",
        SETTINGS => "Filled.Settings",
        ICON_CAPS => "Filled.KeyboardCapslock",
        SPACE => "Filled.SpaceBar",
        UNDO => "AutoMirrored.Filled.Undo",
        REDO => "AutoMirrored.Filled.Redo",
        TOGGLE_ACTIONS_OVERFLOW => "Filled.MoreHoriz",
        TOGGLE_INCOGNITO_MODE => WATERMARK,
        ICON_INCOGNITO_OFF => "drawable.ic_incognito_off",
        TOGGLE_AUTOCORRECT => "Filled.FontDownload",
        CHAR_WIDTH_FULL => "drawable.ic_keyboard_char_width_switcher_full",
        CHAR_WIDTH_HALF => "drawable.ic_keyboard_char_width_switcher_half",
        NOOP => "Filled.Close",
        ICON_UNFOLD_LESS => "Filled.UnfoldLess",
        ICON_UNFOLD_MORE => "Filled.UnfoldMore",
        ICON_STOP => "Filled.Stop",
        ICON_EMAIL => "Filled.Email",
        ICON_LINK => "Filled.Link",
        ICON_PHONE => "Filled.Phone",
        ICON_IMAGE => "Filled.Image",
        ICON_VIDEO => "Filled.Videocam",
        _ => return None,
    })
}

/// The token a Smartbar action draws for its `SmartbarIcon`, with its key's own token for `KEY`.
pub fn smartbar_token(kind: i32, key: i32) -> i32 {
    match kind {
        icon::KEY => key,
        icon::ARROW_LEFT => ARROW_LEFT,
        icon::ARROW_RIGHT => ARROW_RIGHT,
        icon::UNFOLD_LESS => ICON_UNFOLD_LESS,
        icon::UNFOLD_MORE => ICON_UNFOLD_MORE,
        icon::INCOGNITO => TOGGLE_INCOGNITO_MODE,
        icon::STOP => ICON_STOP,
        _ => 0,
    }
}

/// The incognito watermark, which is also the incognito key's and action's icon.
pub const WATERMARK: &str = "drawable.ic_incognito";

/// The emoji panel's category icons in `EmojiCategory` order, then pin, the two page arrows and
/// delete.
pub const EMOJI: [&str; 14] = [
    "Filled.Schedule",
    "Filled.EmojiEmotions",
    "Filled.EmojiPeople",
    "Filled.EmojiNature",
    "Filled.EmojiFoodBeverage",
    "Filled.EmojiTransportation",
    "Filled.EmojiEvents",
    "Filled.EmojiObjects",
    "Filled.EmojiSymbols",
    "Filled.EmojiFlags",
    "Outlined.PushPin",
    "AutoMirrored.Filled.KeyboardArrowLeft",
    "AutoMirrored.Filled.KeyboardArrowRight",
    "Outlined.Delete",
];

/// The clipboard panel's: back, the history switch on and off, clear, filter and no filter,
/// backspace, pin, delete and paste.
pub const CLIPBOARD: [&str; 10] = [
    "AutoMirrored.Filled.ArrowBack",
    "Filled.ToggleOn",
    "Filled.ToggleOff",
    "Filled.DeleteSweep",
    "Filled.FilterList",
    "Filled.FilterListOff",
    "AutoMirrored.Outlined.Backspace",
    "Outlined.PushPin",
    "Filled.Delete",
    "Outlined.ContentPasteGo",
];

/// The one-handed panel's: the keyboard back to full width, then the arrow to its side, start
/// then end.
pub const ONE_HANDED: [[&str; 2]; 2] = [
    ["Filled.ZoomOutMap", "AutoMirrored.Filled.KeyboardArrowLeft"],
    [
        "Filled.ZoomOutMap",
        "AutoMirrored.Filled.KeyboardArrowRight",
    ],
];

/// The subtype sheet's radio buttons, checked then unchecked.
pub const SUBTYPES: [&str; 2] = ["Filled.RadioButtonChecked", "Filled.RadioButtonUnchecked"];

/// The search strip's back and submit.
pub const SEARCH: [&str; 2] = ["AutoMirrored.Filled.ArrowBack", "Filled.Search"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_smartbar_action_draws_its_kinds_token() {
        assert_eq!(smartbar_token(icon::KEY, ICON_SEND), ICON_SEND);
        assert_eq!(smartbar_token(icon::NONE, ICON_SEND), 0);
        assert_eq!(for_key(smartbar_token(icon::INCOGNITO, 0)), Some(WATERMARK));
        assert_eq!(for_key(smartbar_token(icon::STOP, 0)), Some("Filled.Stop"));
    }
}
