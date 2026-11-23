//! The code editor's key data preview, TextKeyDataPreviewBox: the key a code names (a character,
//! a predefined key, or `unspecified` for anything else, as `getCodeInfoAsTextKeyData` gave),
//! labelled as the keyboard labels it in the numeric advanced mode the box's evaluator used, its
//! name and its kind. A character's Unicode name and mark category, and a key's icon, are ICU's and
//! the keyboard's, which Kotlin answers for the one code shown (`wire::settings_ui::theme_codes`).
use crate::settings_ui::App;
use keys::{
    codes::*,
    model_data::{Key, PREDEFINED},
    models,
};
use snygg::text;

/// KeyType's entries by ordinal, as `toString` writes them.
pub const KINDS: [&str; 10] = [
    "character",
    "enter_editing",
    "function",
    "lock",
    "modifier",
    "navigation",
    "system_gui",
    "numeric",
    "placeholder",
    "unspecified",
];
/// KeyboardMode.NUMERIC_ADVANCED, the mode the preview's evaluator gave every key.
const MODE: i32 = 5;
/// The labels `computeLabel` takes from resources, by the token the keyboard's rules give.
pub const LABELS: [(i32, &str); 12] = [
    (PHONE_PAUSE, "key__phone_pause"),
    (PHONE_WAIT, "key__phone_wait"),
    (IME_UI_MODE_TEXT, "key__view_characters"),
    (VIEW_CHARACTERS, "key__view_characters"),
    (VIEW_NUMERIC, "key__view_numeric"),
    (VIEW_NUMERIC_ADVANCED, "key__view_numeric"),
    (VIEW_PHONE, "key__view_phone"),
    (VIEW_PHONE2, "key__view_phone2"),
    (VIEW_SYMBOLS, "key__view_symbols"),
    (VIEW_SYMBOLS2, "key__view_symbols2"),
    (HALF_SPACE, "key__view_half_space"),
    (KESHIDA, "key__view_keshida"),
];

/// The key a code names: its code, KeyType ordinal and label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyData {
    pub code: i32,
    pub kind: i32,
    pub label: String,
}

/// What Kotlin said of a code: its Unicode name, whether it is a non-spacing mark, and the
/// keyboard's icon for it as path data, drawn from the token Kotlin named. Empty until Kotlin
/// answers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Described {
    pub code: i32,
    pub name: String,
    pub non_spacing: bool,
    pub icon: String,
}

/// What the preview shows: the label (None for none, or when the icon shows), the icon's path
/// (empty for none), the name and the kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preview {
    pub label: Option<String>,
    pub icon: String,
    pub name: String,
    pub kind: &'static str,
}

/// `TextKeyData.UNSPECIFIED`.
const UNKNOWN: Key = Key {
    code: UNSPECIFIED,
    kind: 9,
    label: "unspecified",
    discoverable: true,
};

fn predefined(code: i32) -> KeyData {
    let k = PREDEFINED
        .iter()
        .find(|k| k.code == code)
        .unwrap_or(&UNKNOWN);
    KeyData {
        code: k.code,
        kind: k.kind,
        label: k.label.into(),
    }
}

/// The key `code` names, `getCodeInfoAsTextKeyData(code) ?: TextKeyData.UNSPECIFIED`.
pub fn key_data(code: Option<i32>) -> KeyData {
    match code.map(|c| (c, models::code_kind(c))) {
        Some((c, 2)) => KeyData {
            code: c,
            kind: 0,
            label: String::from_utf16_lossy(&models::multi_text(&[c], &[], false)),
        },
        Some((c, 1)) => predefined(c),
        _ => predefined(UNSPECIFIED),
    }
}

/// The code the preview shows for the field's text: the last entry, the one being typed.
pub fn shown_code(field: &str) -> Option<i32> {
    let last = field.rsplit(',').next().unwrap_or("").trim();
    last.parse().ok()
}

impl App {
    /// The preview of key `d`, with what Kotlin said of it when that was about `d`.
    pub fn key_preview(&self, d: &KeyData, said: Option<&Described>) -> Preview {
        let said = said.filter(|s| s.code == d.code);
        let icon = said.map_or(String::new(), |s| s.icon.clone());
        let label = match input::visual::label(d.kind, d.code, MODE) {
            _ if !icon.is_empty() => None,
            0 => None,
            1 => {
                let marked = said.is_some_and(|s| s.non_spacing);
                let shown = models::text(d.code, &text(&d.label), true, marked);
                Some(String::from_utf16_lossy(&shown))
            }
            token => LABELS
                .iter()
                .find(|(c, _)| *c == token)
                .map(|(_, key)| self.label(key)),
        };
        let name = if d.code > 0 {
            said.map_or(String::new(), |s| s.name.clone())
        } else {
            d.label.clone()
        };
        let kind = usize::try_from(d.kind).ok().and_then(|k| KINDS.get(k));
        Preview {
            label,
            icon,
            name,
            kind: kind.copied().unwrap_or("unspecified"),
        }
    }
}
