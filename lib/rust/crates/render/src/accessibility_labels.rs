//! Compose QuickActionButton/computeDisplayName is the spec for smartbar names.
//! Keys have no Compose semantic activation; use LatinIME KeyCodeDescriptionMapper conventions.
use keys::codes::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct Labels(pub BTreeMap<String, String>);
impl Labels {
    pub fn get(&self, name: &str, fallback: &str) -> String {
        self.0
            .get(name)
            .map(String::as_str)
            .unwrap_or(fallback)
            .into()
    }
    pub fn shift(&self, shift: &str) -> String {
        match shift {
            "caps_lock" => self.get("access_caps", "Caps lock enabled"),
            "shifted_manual" | "shifted_automatic" => self.get("access_shift_on", "Shift enabled"),
            _ => self.get("access_shift", "Shift"),
        }
    }
    pub fn mode(&self, mode: &str) -> String {
        match mode {
            "symbols" => self.get("access_symbols", "Symbols"),
            "symbols2" => self.get("access_symbols2", "More symbols"),
            "numeric" | "numeric_advanced" => self.get("access_numeric", "Numeric"),
            "phone" | "phone2" => self.get("access_phone", "Phone"),
            _ => self.get("access_letters", "Letters"),
        }
    }
    pub fn key(&self, code: i32, label: &str, shift: &str) -> String {
        let (name, fallback) = match code {
            SHIFT => return self.shift(shift),
            CAPS_LOCK => ("access_caps", "Caps lock enabled"),
            DELETE | DELETE_WORD => ("access_delete", "Delete"),
            SPACE | CJK_SPACE => ("access_space", "Space"),
            ENTER if label.trim().is_empty() || label == "enter" => ("access_return", "Return"),
            TAB => ("access_tab", "Tab"),
            VIEW_CHARACTERS => ("access_letters", "Letters"),
            VIEW_SYMBOLS => ("access_symbols", "Symbols"),
            VIEW_SYMBOLS2 => ("access_symbols2", "More symbols"),
            VIEW_NUMERIC | VIEW_NUMERIC_ADVANCED => ("access_numeric", "Numeric"),
            VIEW_PHONE | VIEW_PHONE2 => ("access_phone", "Phone"),
            CTRL | CTRL_LOCK => ("access_control", "Control"),
            ALT | ALT_LOCK => ("access_alt", "Alt"),
            FN | FN_LOCK => ("access_function", "Function"),
            ESCAPE => ("access_escape", "Escape"),
            _ => {
                if code < 0 && code != MULTIPLE_CODE_POINTS && code != URI_COMPONENT_TLD {
                    return self.action(code, label, 0);
                }
                return self.symbol(label);
            }
        };
        self.get(name, fallback)
    }
    /// Same no-enter/multiline and IME action rules as ComputingEvaluator's Enter icon.
    pub fn enter(&self, options: i32, input_type: i32) -> String {
        let (name, fallback) = if options & 0x4000_0000 != 0 || input_type & 0x0002_0000 != 0 {
            ("access_return", "Return")
        } else {
            match options & 255 {
                2 => ("access_go", "Go"),
                3 => ("access_search", "Search"),
                4 => ("access_send", "Send"),
                5 => ("access_next", "Next"),
                6 => ("access_done", "Done"),
                7 => ("access_previous", "Previous"),
                _ => ("access_return", "Return"),
            }
        };
        self.get(name, fallback)
    }
    pub fn symbol(&self, label: &str) -> String {
        let fallback = match label {
            "!" => "Exclamation mark",
            "\"" => "Quotation mark",
            "#" => "Number sign",
            "$" => "Dollar sign",
            "%" => "Percent",
            "&" => "Ampersand",
            "'" => "Apostrophe",
            "(" => "Left parenthesis",
            ")" => "Right parenthesis",
            "*" => "Asterisk",
            "+" => "Plus",
            "," => "Comma",
            "-" => "Hyphen",
            "." => "Period",
            "/" => "Slash",
            ":" => "Colon",
            ";" => "Semicolon",
            "<" => "Less than",
            "=" => "Equals",
            ">" => "Greater than",
            "?" => "Question mark",
            "@" => "At sign",
            "[" => "Left bracket",
            "\\" => "Backslash",
            "]" => "Right bracket",
            "^" => "Caret",
            "_" => "Underscore",
            "`" => "Grave accent",
            "{" => "Left brace",
            "|" => "Vertical bar",
            "}" => "Right brace",
            "~" => "Tilde",
            "€" => "Euro",
            "£" => "Pound sterling",
            "¥" => "Yen",
            "¢" => "Cent",
            "°" => "Degree",
            "×" => "Multiplication sign",
            "÷" => "Division sign",
            _ => return label.into(),
        };
        let cp = label.chars().next().unwrap_or_default() as u32;
        self.get(&format!("access_symbol_{cp:04x}"), fallback)
    }
    pub fn action(&self, code: i32, text: &str, lit: i32) -> String {
        if code == VOICE_INPUT && lit == 1 {
            return self.get("dictation__stop", "Stop dictating");
        }
        if code == READ_ALOUD && lit == 2 {
            return self.get("read_aloud__stop", "Stop reading");
        }
        let (suffix, fallback) = match code {
            ARROW_UP => ("arrow_up", "Arrow up"),
            ARROW_DOWN => ("arrow_down", "Arrow down"),
            ARROW_LEFT => ("arrow_left", "Arrow left"),
            ARROW_RIGHT => ("arrow_right", "Arrow right"),
            CLIPBOARD_CLEAR_PRIMARY_CLIP => (
                "clipboard_clear_primary_clip",
                "Clear primary clipboard clip",
            ),
            CLIPBOARD_COPY => ("clipboard_copy", "Copy"),
            CLIPBOARD_CUT => ("clipboard_cut", "Cut"),
            CLIPBOARD_PASTE => ("clipboard_paste", "Paste"),
            CLIPBOARD_SELECT_ALL => ("clipboard_select_all", "Select all"),
            FORWARD_DELETE => ("forward_delete", "Forward delete"),
            IME_UI_MODE_CLIPBOARD => ("ime_ui_mode_clipboard", "Clipboard"),
            IME_UI_MODE_MEDIA => ("ime_ui_mode_media", "Emoji"),
            LANGUAGE_SWITCH => ("language_switch", "Switch language"),
            SETTINGS => ("settings", "Settings"),
            UNDO => ("undo", "Undo"),
            REDO => ("redo", "Redo"),
            TOGGLE_ACTIONS_OVERFLOW => ("toggle_actions_overflow", "More actions"),
            TOGGLE_INCOGNITO_MODE => ("toggle_incognito_mode", "Incognito"),
            TOGGLE_AUTOCORRECT => ("toggle_autocorrect", "Autocorrect"),
            VOICE_INPUT => ("voice_input", "Voice input"),
            READ_ALOUD => ("read_aloud", "Read aloud"),
            IME_HIDE_UI => ("ime_hide_ui", "Hide keyboard"),
            TOGGLE_COMPACT_LAYOUT => ("one_handed_mode", "One-handed"),
            NOOP => ("noop", "None"),
            DRAG_MARKER => return String::new(),
            _ => return text.into(),
        };
        self.get(&format!("quick_action__{suffix}"), fallback)
    }
}
