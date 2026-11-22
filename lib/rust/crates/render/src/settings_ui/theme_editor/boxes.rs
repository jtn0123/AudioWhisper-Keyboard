//! A rule's box: SnyggRuleRow's lines (the element, its selector on primaryContainer, each
//! attribute as `key = [values]` at 56%) with Duplicate and Edit at the end, `@defines`' note,
//! then the properties as ListItems of name over value with a Delete each. A rule with several
//! property sets (`@font`) heads each with "Source set", as Compose did. A font set's `src`,
//! which the stylesheet cannot be written without, has no Delete.
use super::{Gone, Tap, paint::rule_id};
use crate::settings_ui::{Action, App, cards::plain, items::Entry};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;
use snygg::{grammar::Rule, resolve::PropertySets};

/// Whether property `name` of `rule` must stay: a font set's `src`.
pub fn required(rule: &Rule, name: &str) -> bool {
    matches!(rule, Rule::Font(_)) && name == "src"
}

impl App {
    /// Rule `r`'s box, 16 dp in and 8 dp round; its height with that margin.
    pub(super) fn rule_box(
        &mut self,
        p: &mut Panel,
        r: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let left = x + self.px(16.0);
        let width = self.config[0] - x - 2.0 * self.px(16.0);
        let top = y + self.px(8.0);
        let outline = p.items.len();
        self.outlined_box(p, rect(left, top, width, 0.0), clip);
        let mut at = top + self.rule_row(p, r, (left, top, width), clip, text);
        let (rule, sets) = self.theme_editor.rules[r].clone();
        if rule == Rule::Defines {
            let note = self.label("snygg__rule_annotation__defines_description");
            let ink = self.colors[2];
            at += self.theme_note(p, (rule_id(r, 3), note, ink, 0.0), (left, at), clip, text);
            at += self.px(8.0);
        }
        let multiple = matches!(sets, PropertySets::Multiple(_));
        let gone = match &self.theme_editor.deleted {
            Some(Gone::Property(g, s, k, ..)) if *g == rule => Some((*s, *k)),
            _ => None,
        };
        let props = self.rule_properties(r);
        // Undo stands where the property was: before the one now in its place.
        let undo_at = gone.map(|(s, k)| {
            let after = props
                .iter()
                .position(|q| (q.set == s && q.place >= k) || q.set > s);
            after.unwrap_or(props.len())
        });
        for (n, q) in props.iter().cloned().enumerate() {
            if multiple && q.place == 0 {
                let ink = self.colors[1];
                let heading = (
                    rule_id(r, 4 + q.set),
                    "Source set".into(),
                    ink,
                    self.px(8.0),
                );
                at += self.theme_note(p, heading, (left, at), clip, text);
            }
            if undo_at == Some(n) {
                at += self.theme_undo(p, (left, at, width), clip, text);
            }
            let keep = required(&rule, &q.raw);
            let place = (r, n, q.set, q.place);
            at += self.property_row(p, place, (q.name, q.value, !keep), (left, at), clip, text);
        }
        if undo_at == Some(props.len()) {
            at += self.theme_undo(p, (left, at, width), clip, text);
        }
        self.fit_box(p, outline, at - top);
        at + self.px(8.0) - y
    }
    /// SnyggRuleRow: the rule's lines 16 dp in, its buttons 6 dp from the end; its height.
    fn rule_row(
        &mut self,
        p: &mut Panel,
        r: usize,
        (left, top, width): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let rule = self.theme_editor.rules[r].0.clone();
        let (title, chip, attrs) = self.rule_lines(&rule);
        let defines = rule == Rule::Defines;
        let buttons = if defines { 0.0 } else { 2.0 * self.px(48.0) };
        let room = (width - self.px(16.0) - buttons - self.px(6.0)).max(1.0);
        let shown = self.row_text(text, &title, self.sp(14), room, 2);
        let th = self.measure(text, &shown, self.sp(14), room).ceil();
        let ink = self.colors[1];
        let at = rect(left + self.px(16.0), top + self.px(8.0), room, th);
        self.paint_text(p, &shown, at, self.sp(14), ink, clip);
        let mut y = at.bottom;
        if !chip.is_empty() {
            y += self.selector_tag(p, &chip, (at.left, y), clip, text);
        }
        let faded = (self.colors[1] & 0x00ff_ffff) | 0x8f00_0000;
        for line in &attrs {
            let lh = self.measure(text, line, self.sp(14), room).ceil();
            self.paint_text(
                p,
                line,
                rect(at.left, y, room, lh),
                self.sp(14),
                faded,
                clip,
            );
            y += lh;
        }
        let rh = (y + self.px(8.0) - top).max(self.px(48.0));
        let row = rect(left, top, width - buttons, rh);
        if self.focus == rule_id(r, 0) {
            self.paint_clipped_box(p, row, self.colors[4], self.px(8.0), clip);
        }
        let mut c = plain(rule_id(r, 0), row, title.clone(), "button");
        c.summary = [chip, attrs.join("\n")].join("\n").trim().to_owned();
        if defines {
            c.role = "text";
        } else {
            c.action = Action::ThemeEdit(Tap::Edit(r));
        }
        self.push_control(c, clip);
        if !defines {
            let b = rect(left, top, width - self.px(2.0), rh.min(self.px(56.0)));
            let pairs = [
                (1, "content_copy", "settings__theme_editor__duplicate_rule"),
                (2, "edit", "settings__theme_editor__edit_rule"),
            ];
            for (part, icon, key) in pairs {
                let button = self.end_button(b, 3 - part);
                let id = rule_id(r, part);
                self.paint_button_icon(p, id, icon, button, self.colors[3], clip);
                let mut c = plain(id, button, self.label(key), "button");
                c.summary = title.clone();
                c.action = Action::ThemeEdit(if part == 1 {
                    Tap::Duplicate(r)
                } else {
                    Tap::Edit(r)
                });
                self.push_control(c, clip);
            }
        }
        rh
    }
    /// The selector on primaryContainer in small corners; its height with 4 dp under it.
    fn selector_tag(
        &mut self,
        p: &mut Panel,
        chip: &str,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let size = self.sp(14);
        let layout = text.layout_typography(chip, size, false, 400, false, [0.0, 0.0]);
        let (cw, ch) = (layout.width().ceil(), layout.height().ceil());
        let tag = rect(x, y + self.px(4.0), cw + 2.0 * self.px(4.0), ch);
        self.paint_clipped_box(p, tag, self.colors[4], self.px(4.0), clip);
        let at = rect(tag.left + self.px(4.0), tag.top, cw + 1.0, ch);
        self.paint_text(p, chip, at, size, self.colors[14], clip);
        ch + self.px(8.0)
    }
    /// Property `k` of set `s` of rule `r`, the rule's `n`th: its name over its value, and
    /// Delete when it may go; its height.
    fn property_row(
        &mut self,
        p: &mut Panel,
        (r, n, s, k): (usize, usize, usize, usize),
        (name, value, deletable): (String, String, bool),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let entry = Entry {
            id: rule_id(r, 64 + 2 * n),
            title: name.clone(),
            summary: value,
            icon: "",
            ink: [None, None],
            buttons: 1,
            right: self.px(16.0),
            enabled: true,
            action: Action::None,
        };
        let (rh, b) = self.paint_entry(p, entry, at, clip, text);
        if deletable {
            let id = rule_id(r, 65 + 2 * n);
            let button = self.end_button(b, 1);
            self.paint_button_icon(p, id, "delete", button, self.colors[2], clip);
            let mut c = plain(id, button, self.label("action__delete"), "button");
            c.summary = name;
            c.action = Action::ThemeEdit(Tap::DeleteProperty(r, s, k));
            self.push_control(c, clip);
        }
        rh
    }
    /// The rules' focus order: Add rule, then each rule's row, Duplicate and Edit, each property's
    /// Delete, and Undo where it stands.
    pub(super) fn rules_focus(&self) -> Vec<i32> {
        let mut out = vec![super::paint::ADD_ID];
        for (r, (rule, _)) in self.theme_editor.rules.iter().enumerate() {
            if *rule != Rule::Defines {
                out.extend([rule_id(r, 0), rule_id(r, 1), rule_id(r, 2)]);
            }
            let props = self.rule_properties(r);
            for (n, q) in props.iter().enumerate() {
                if !required(rule, &q.raw) {
                    out.push(rule_id(r, 65 + 2 * n));
                }
            }
        }
        if self.theme_editor.deleted.is_some() {
            out.push(crate::settings_ui::items::UNDO_ID);
        }
        out
    }
}
