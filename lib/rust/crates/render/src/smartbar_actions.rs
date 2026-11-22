//! What each smartbar action shows: its icon, its label and its state, and what the arrow
//! toggles. Android sends what its key evaluator knows of each action (`wire::smartbar`) and
//! draws the icon each answer names; every choice between them is made here.

use crate::protocol::{
    BAR_DICTATING, BAR_ENABLED, BAR_FINISHING, BAR_LEVEL_MASK, BAR_LEVEL_SHIFT, BAR_READING,
    BAR_WARMING,
};
use crate::smartbar::Config;
use keys::codes::{READ_ALOUD, VOICE_INPUT};
use wire::smartbar::{action, bar, icon, kind, plan, shown};

/// `SmartbarLayout.SUGGESTIONS_ACTIONS_EXTENDED`'s ordinal, as `smartbar::Config` reads it.
const EXTENDED_LAYOUT: i32 = 3;
/// `IncognitoDisplayMode.REPLACE_SHARED_ACTIONS_TOGGLE`'s ordinal: incognito takes the arrow's
/// place.
const REPLACES_TOGGLE: i32 = 0;
/// `IncognitoDisplayMode.DISPLAY_BEHIND_KEYBOARD`'s ordinal: incognito shows behind the keys.
const BEHIND_KEYS: i32 = 1;
/// Where the packed config keeps the shared actions' setting, as `Config::decode` reads it.
const SHARED_SETTING: usize = 3;
/// What a text action shows when its text starts with nothing it can draw.
const NO_TEXT: u16 = b'?' as u16;

/// The facts every action reads (`wire::smartbar::bar`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bar {
    /// The `SmartbarLayout` ordinal.
    pub layout: i32,
    /// The arrow points the other way.
    pub flip: bool,
    /// The shared actions are expanded.
    pub shared: bool,
    /// The extended actions are expanded.
    pub extended: bool,
    /// The `IncognitoDisplayMode` ordinal.
    pub incognito_display: i32,
    /// Incognito is on.
    pub incognito: bool,
    /// Dictation is running: listening or finishing.
    pub dictating: bool,
    /// Dictation is listening.
    pub listening: bool,
    /// The dictation model is warming up.
    pub warming: bool,
    /// The microphone's level, from 0 to 1.
    pub level: f32,
    /// Reading aloud.
    pub reading: bool,
}

/// One action, as Android's key evaluator sees it (`wire::smartbar::action`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action<'a> {
    /// A `SmartbarActionKind`.
    pub kind: i32,
    /// Its key's code.
    pub code: i32,
    /// The evaluator enables its key.
    pub enabled: bool,
    /// Its key has an icon of its own.
    pub key_icon: bool,
    /// The text it types.
    pub text: &'a [u16],
}

/// What one action shows (`wire::smartbar::shown`).
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    /// The code the renderer reports a tap on.
    pub code: i32,
    /// The `RenderProtocol` `BAR_` flags and the microphone level.
    pub flags: i32,
    /// A `SmartbarIcon`.
    pub icon: i32,
    /// Its own label, when it shows one.
    pub label: Vec<u16>,
    /// Show the key's own label instead, which only Android's evaluator can name.
    pub key_label: bool,
}

/// What the smartbar shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    /// The actions show expanded, so the arrow collapses them.
    pub expanded: bool,
    /// The arrow flips the extended actions, else the shared ones.
    pub toggles_extended: bool,
    /// What each action shows, in order.
    pub actions: Vec<Shown>,
}

impl Bar {
    fn extended_layout(&self) -> bool {
        self.layout == EXTENDED_LAYOUT
    }

    /// Whether the actions the arrow toggles are expanded.
    pub fn expanded(&self) -> bool {
        if self.extended_layout() {
            self.extended
        } else {
            self.shared
        }
    }

    /// The arrow points left when flipped, or when expanded actions show behind the keys, but
    /// not both.
    pub fn arrow(&self) -> i32 {
        if self.flip ^ (self.expanded() && self.incognito_display == BEHIND_KEYS) {
            icon::ARROW_LEFT
        } else {
            icon::ARROW_RIGHT
        }
    }

    /// The toggle's icon: unfold in the extended layout, else incognito in the arrow's place,
    /// else the arrow.
    pub fn toggle(&self) -> i32 {
        if self.extended_layout() {
            if self.extended {
                icon::UNFOLD_LESS
            } else {
                icon::UNFOLD_MORE
            }
        } else if self.incognito && self.incognito_display == REPLACES_TOGGLE {
            icon::INCOGNITO
        } else {
            self.arrow()
        }
    }

    /// An action's flags: enabled, lit while it dictates or reads, and the microphone's state.
    fn flags(&self, action: &Action) -> i32 {
        let code = code(action);
        let mut flags = 0;
        if action.kind == kind::NONE || action.enabled {
            flags |= BAR_ENABLED;
        }
        if code == VOICE_INPUT {
            flags |= self.voice();
        }
        if code == READ_ALOUD && self.reading {
            flags |= BAR_READING;
        }
        flags
    }

    /// The microphone key's flags and level.
    fn voice(&self) -> i32 {
        let mut flags = (self.level.clamp(0.0, 1.0) * BAR_LEVEL_MASK as f32) as i32;
        flags <<= BAR_LEVEL_SHIFT;
        if self.dictating {
            flags |= BAR_DICTATING;
        }
        if self.warming {
            flags |= BAR_WARMING;
        }
        if !self.listening {
            flags |= BAR_FINISHING;
        }
        flags
    }

    /// An action's icon: stop while it reads aloud, the toggle's in an empty slot, else its key's.
    fn icon(&self, action: &Action) -> i32 {
        if code(action) == READ_ALOUD && self.reading {
            icon::STOP
        } else if action.kind == kind::NONE {
            self.toggle()
        } else if action.kind == kind::KEY && action.key_icon {
            icon::KEY
        } else {
            icon::NONE
        }
    }

    /// What `action` shows.
    pub fn shown(&self, action: &Action) -> Shown {
        let icon = self.icon(action);
        Shown {
            code: code(action),
            flags: self.flags(action),
            icon,
            label: if action.kind == kind::TEXT {
                text_label(action.text)
            } else {
                Vec::new()
            },
            key_label: action.kind == kind::KEY && icon == icon::NONE,
        }
    }
}

/// The code a tap reports: none for an empty slot.
fn code(action: &Action) -> i32 {
    if action.kind == kind::NONE {
        0
    } else {
        action.code
    }
}

/// A text action's label: the first character of its text, a whole surrogate pair when it starts
/// with one, or `?` when the text is empty or starts with whitespace (Kotlin's
/// `Char.isWhitespace`).
pub fn text_label(text: &[u16]) -> Vec<u16> {
    let paired = matches!(text, [high, low, ..]
        if (0xD800..0xDC00).contains(high) && (0xDC00..0xE000).contains(low));
    let first = &text[..text.len().min(if paired { 2 } else { 1 })];
    if chars::is_blank(first) {
        vec![NO_TEXT]
    } else {
        first.to_vec()
    }
}

/// What the smartbar shows for `bar` and its `actions`.
pub fn plan(bar: &Bar, actions: &[Action]) -> Plan {
    Plan {
        expanded: bar.expanded(),
        toggles_extended: bar.extended_layout(),
        actions: actions.iter().map(|a| bar.shown(a)).collect(),
    }
}

/// The plan for the smartbar's packed `config`, and `fields`: a `SmartbarBar` row and a
/// `SmartbarAction` row for each action. It is packed as a `SmartbarPlan` row and a
/// `SmartbarActionPlan` row for each action. `None` when the config is not one, a row is cut
/// short or a number doesn't parse.
pub fn call(config: &[f32], fields: &[Vec<u16>]) -> Option<Vec<Vec<u16>>> {
    let rows = fields.get(bar::SIZE..)?;
    if !rows.len().is_multiple_of(action::SIZE) {
        return None;
    }
    let bar = decode_bar(config, &fields[..bar::SIZE])?;
    let actions = rows
        .chunks(action::SIZE)
        .map(decode_action)
        .collect::<Option<Vec<_>>>()?;
    Some(encode(&plan(&bar, &actions)))
}

/// The bar from the config, which says what the layout shows, and the row, which says the rest.
/// The shared actions' setting is read as it is set: a preview shows them folded whatever it says
/// (`smartbar::Config::shared`), but the arrow still follows it.
fn decode_bar(config: &[f32], row: &[Vec<u16>]) -> Option<Bar> {
    let c = Config::decode(config)?;
    let flag = |i: usize| wire::flag::decode(&row[i]);
    Some(Bar {
        layout: c.layout,
        flip: c.flip,
        shared: config[SHARED_SETTING] != 0.0,
        extended: c.extended,
        incognito_display: number(&row[bar::INCOGNITO_DISPLAY])?,
        incognito: flag(bar::INCOGNITO),
        dictating: c.dictating,
        listening: flag(bar::LISTENING),
        warming: flag(bar::WARMING),
        level: number(&row[bar::LEVEL])?,
        reading: c.reading,
    })
}

fn decode_action(row: &[Vec<u16>]) -> Option<Action<'_>> {
    Some(Action {
        kind: number(&row[action::KIND])?,
        code: number(&row[action::CODE])?,
        enabled: wire::flag::decode(&row[action::ENABLED]),
        key_icon: wire::flag::decode(&row[action::KEY_ICON]),
        text: &row[action::TEXT],
    })
}

/// A number as Kotlin's `toString` spells it.
fn number<T: std::str::FromStr>(units: &[u16]) -> Option<T> {
    String::from_utf16(units).ok()?.parse().ok()
}

fn encode(plan: &Plan) -> Vec<Vec<u16>> {
    let mut out = vec![Vec::new(); plan::SIZE];
    out[plan::EXPANDED] = wire::flag::utf16(plan.expanded);
    out[plan::TOGGLES_EXTENDED] = wire::flag::utf16(plan.toggles_extended);
    for action in &plan.actions {
        let mut row = vec![Vec::new(); shown::SIZE];
        row[shown::CODE] = text(action.code);
        row[shown::FLAGS] = text(action.flags);
        row[shown::ICON] = text(action.icon);
        row[shown::LABEL].clone_from(&action.label);
        row[shown::KEY_LABEL] = wire::flag::utf16(action.key_label);
        out.extend(row);
    }
    out
}

fn text(value: i32) -> Vec<u16> {
    value.to_string().encode_utf16().collect()
}
