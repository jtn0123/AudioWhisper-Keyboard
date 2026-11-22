//! A `@font` rule's source sets, as ThemeEditorScreen drew them: each headed "Source set" with
//! Up, Down, Delete (undoable here, as a property's Delete is) and Add (a property to the set),
//! and the rule's Add appending an empty set. Under a rule's title or a set's heading, the
//! "Errors" box names each property the spec requires that the set lacks: a font set's `src`.
use super::{Gone, Rules, Tap, property::PropTap};
use crate::settings_ui::{Action, App, cards::plain};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;
use snygg::{
    Text as Name,
    grammar::Rule,
    resolve::PropertySets,
    spec::{self, Property},
    text,
    value::Value,
};

/// Set `s` of rule `r`'s controls start at `SET_ID + 65_536 * r + 8 * s`: 0 its heading, 1 Up,
/// 2 Down, 3 Delete, 4 Add, 5 the Errors box's message and 6 its title.
pub const SET_ID: i32 = 100_000_000;
/// The Errors box's title and message.
pub const ERRORS: &str = "settings__theme_editor__property_errors";
pub const REQUIRED: &str = "settings__theme_editor__property_required";

pub fn set_id(r: usize, s: usize, part: usize) -> i32 {
    SET_ID + 65_536 * r as i32 + 8 * s.min(8191) as i32 + part.min(7) as i32
}
/// The properties the spec requires of a set of `rule`.
fn required_of(rule: &Rule) -> Vec<&'static Property> {
    let all = match rule {
        Rule::Defines => &[][..],
        Rule::Font(_) => spec::fonts(),
        Rule::Element { .. } => spec::elements(),
    };
    all.iter().filter(|p| p.required).collect()
}
/// The required properties `props` lacks.
pub fn missing(rule: &Rule, props: &[(Name, Value)]) -> Vec<&'static str> {
    let has = |name: &str| props.iter().any(|(n, _)| *n == text(name));
    required_of(rule)
        .into_iter()
        .map(|p| p.name)
        .filter(|n| !has(n))
        .collect()
}
/// A deleted set back at `s` of its rule, unless the rule is gone.
pub fn put_back(rules: &mut Rules, rule: &Rule, s: usize, set: Vec<(Name, Value)>) -> bool {
    match rules.iter_mut().find(|(r, _)| r == rule) {
        Some((_, PropertySets::Multiple(sets))) => {
            sets.insert(s.min(sets.len()), set);
            true
        }
        _ => false,
    }
}

impl App {
    fn sets_mut(&mut self, r: usize) -> Option<&mut Vec<Vec<(Name, Value)>>> {
        match self.theme_editor.rules.get_mut(r)? {
            (_, PropertySets::Multiple(sets)) => Some(sets),
            _ => None,
        }
    }
    /// `@font`'s Add source set: an empty set at the end.
    pub(super) fn add_set(&mut self, r: usize) {
        if let Some(sets) = self.sets_mut(r) {
            sets.push(vec![]);
            self.theme_changed();
        }
    }
    /// Set `s` of rule `r` one up, or one down.
    pub(super) fn move_set(&mut self, r: usize, s: usize, down: bool) {
        let Some(sets) = self.sets_mut(r) else {
            return;
        };
        let to = if down { s + 1 } else { s.wrapping_sub(1) };
        if s < sets.len() && to < sets.len() {
            sets.swap(s, to);
            self.theme_changed();
        }
    }
    /// Deletes set `s` of rule `r`; Undo puts it back.
    pub(super) fn delete_set(&mut self, r: usize, s: usize) {
        let Some((rule, PropertySets::Multiple(sets))) = self.theme_editor.rules.get_mut(r) else {
            return;
        };
        if s < sets.len() {
            let set = sets.remove(s);
            self.theme_editor.deleted = Some(Gone::Set(rule.clone(), s, set));
            self.theme_changed();
        }
    }
    /// Set `s` of `count` in rule `r`'s box: "Source set", then Up, Down, Delete and Add at the
    /// end; its height.
    pub(super) fn set_heading(
        &mut self,
        p: &mut Panel,
        (r, s, count): (usize, usize, usize),
        (left, y, width): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let rh = self.px(48.0);
        let room = (width - self.px(16.0) - 4.0 * self.px(48.0) - self.px(6.0)).max(1.0);
        let label = String::from("Source set");
        let th = self.measure(text, &label, self.sp(14), room).ceil();
        let at = rect(
            left + self.px(16.0),
            y + ((rh - th) / 2.0).round(),
            room,
            th,
        );
        self.paint_text(p, &label, at, self.sp(14), self.colors[1], clip);
        self.push_control(plain(set_id(r, s, 0), at, label.clone(), "heading"), clip);
        let b = rect(left, y, width - self.px(2.0), rh);
        let buttons = [
            (1, "arrow_upward", "settings__theme_editor__move_up", s > 0),
            (
                2,
                "arrow_downward",
                "settings__theme_editor__move_down",
                s + 1 < count,
            ),
            (3, "delete", "action__delete", true),
            (4, "add", "settings__theme_editor__add_property", true),
        ];
        for (part, icon, key, enabled) in buttons {
            let tap = match part {
                1 => PropTap::Up(r, s),
                2 => PropTap::Down(r, s),
                3 => PropTap::DeleteSet(r, s),
                _ => PropTap::AddTo(r, s),
            };
            let (id, button) = (set_id(r, s, part), self.end_button(b, 5 - part));
            let ink = self.enabled_color(self.colors[3], enabled);
            self.paint_button_icon(p, id, icon, button, ink, clip);
            let mut c = plain(id, button, self.label(key), "button");
            (c.summary, c.enabled) = (label.clone(), enabled);
            c.action = Action::ThemeEdit(Tap::Prop(tap));
            self.push_control(c, clip);
        }
        rh
    }
    /// The Errors box of set `s` of rule `r`: an outlined box titled "Errors" naming each
    /// required property the set lacks, in the error colour; its height, nothing if none.
    pub(super) fn errors_box(
        &mut self,
        p: &mut Panel,
        (r, s): (usize, usize),
        (left, y, width): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let (rule, sets) = &self.theme_editor.rules[r];
        let props = super::property::set_ref(sets, s).map_or(&[][..], |p| &p[..]);
        let lacking = missing(rule, props);
        if lacking.is_empty() {
            return 0.0;
        }
        let message = lacking
            .iter()
            .map(|name| self.label(REQUIRED).replace("{name}", name))
            .collect::<Vec<_>>()
            .join("\n");
        let (inner, w) = (left + self.px(8.0), width - 2.0 * self.px(8.0));
        let border = rect(inner, y + self.px(12.0), w, 0.0);
        let outline = p.items.len();
        self.outlined_box(p, border, clip);
        let ink = self.colors[18];
        let note = (set_id(r, s, 5), message, ink, 0.0);
        let at = border.top + self.px(12.0);
        let bottom = at + self.theme_note(p, note, (inner, at), clip, text) + self.px(8.0);
        self.fit_box(p, outline, bottom - border.top);
        let title = self.label(ERRORS);
        self.box_title(
            p,
            (set_id(r, s, 6), &title),
            Action::None,
            border,
            clip,
            text,
        );
        bottom + self.px(8.0) - y
    }
    /// The focus order of set `s` of `count` in rule `r`: its heading's buttons but a disabled
    /// Up or Down.
    pub(super) fn set_focus(&self, r: usize, s: usize, count: usize) -> Vec<i32> {
        let mut out = vec![];
        if s > 0 {
            out.push(set_id(r, s, 1));
        }
        if s + 1 < count {
            out.push(set_id(r, s, 2));
        }
        out.extend([set_id(r, s, 3), set_id(r, s, 4)]);
        out
    }
}
