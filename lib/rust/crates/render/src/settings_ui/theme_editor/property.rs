//! The property editor, EditPropertyDialog as a screen: the name (a dropdown of the spec's
//! properties for an element or `@font`, a field for a variable, fixed when editing as Compose
//! disabled it), the encoder (choosing one starts from its default value, and a new name clears
//! it, as Compose did), and the value as its encoder edits it (`property_values.rs`). Apply
//! refuses with no name or encoder, a value the stylesheet cannot hold, or, adding, a name the
//! set has already, which Compose meant to refuse and silently overwrote. Delete deletes through
//! the rules' Delete, so Undo puts it back.
//!
//! snygg v2's spec gives a property no level (Compose's `propertySpec.level <= snyggLevel`
//! filter is commented out in ThemeEditorScreen), so the editor level changes only how names
//! are shown, never which are offered.
use super::{
    PROPERTY_ADD, PROPERTY_EDIT, Rules,
    property_values::{self as values, Editing, editing},
};
use crate::settings_ui::{
    App,
    catalogue::{Kind, theme_editor::Part},
};
use snygg::{
    Text,
    grammar::Rule,
    resolve::PropertySets,
    spec, text,
    value::{COLOR_NAMES, ENCODERS, Encoder, Value},
};

/// Why Apply refused a name the set has.
pub const EXISTS: &str = "settings__theme_editor__property_already_exists";
/// The strings the property editor shows beyond its rows'.
pub const KEYS: [&str; 2] = [EXISTS, "settings__theme_editor__padding_apply_for_all"];
/// Padding's sides' names, in `value::PADDING_IDS` order, and the corners', `CORNER_IDS` order.
pub const SIDES: [&str; 4] = [
    "settings__theme_editor__padding_start",
    "settings__theme_editor__padding_top",
    "settings__theme_editor__padding_end",
    "settings__theme_editor__padding_bottom",
];
pub const CORNERS: [&str; 4] = [
    "enum__shape_corner__top_start",
    "enum__shape_corner__top_end",
    "enum__shape_corner__bottom_end",
    "enum__shape_corner__bottom_start",
];

/// Every string key the property editor shows beyond its rows'.
pub fn string_keys() -> Vec<&'static str> {
    let mut out = KEYS.to_vec();
    out.extend(SIDES.iter().chain(&CORNERS));
    out.extend(values::MESSAGES);
    out.extend(ENCODERS.iter().map(|e| values::encoder_key(*e)));
    out
}

/// A tap on the property editor's controls.
#[derive(Clone, Debug, PartialEq)]
pub enum PropTap {
    /// Rule `r`'s Add property.
    Add(usize),
    /// Property `k` of set `s` of rule `r`: the editor showing it.
    Edit(usize, usize, usize),
    /// The chip copying the side or corner edited last to all four.
    All,
    /// The top bar's Delete.
    Delete,
}

fn lossy(t: &[u16]) -> String {
    String::from_utf16_lossy(t)
}

/// The property editor's state.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyForm {
    /// The rule and the set (0 for a single set) the property is in.
    pub rule: Rule,
    pub set: usize,
    /// The property being changed, by name; None while adding.
    pub original: Option<String>,
    pub name: String,
    /// None for "select".
    pub encoder: Option<Encoder>,
    /// The value dropdown's choice, and the fields' texts (`property_values::texts_of`).
    pub choice: String,
    pub texts: [String; 4],
    /// The side or corner edited last, which the chip copies.
    pub last: usize,
    /// Why Apply refused: a string key, or empty.
    pub error: &'static str,
    /// Apply was tried: an unchosen name or encoder is marked.
    pub checked: bool,
}

impl PropertyForm {
    pub fn adding(rule: Rule, set: usize) -> Self {
        Self {
            rule,
            set,
            original: None,
            name: String::new(),
            encoder: None,
            choice: String::new(),
            texts: Default::default(),
            last: 0,
            error: "",
            checked: false,
        }
    }
    pub fn editing(rule: Rule, set: usize, name: &[u16], value: &Value) -> Self {
        let (choice, texts) = values::texts_of(value);
        Self {
            original: Some(lossy(name)),
            name: lossy(name),
            encoder: Some(value.encoder()),
            choice,
            texts,
            ..Self::adding(rule, set)
        }
    }
    /// The property's key: the name trimmed (Compose kept a variable's spaces, and then
    /// offered it no encoder).
    pub fn key(&self) -> Text {
        text(self.name.trim())
    }
    pub fn variable(&self) -> bool {
        self.rule == Rule::Defines
    }
    /// Whether a name is given: a valid variable name, or any of the spec's.
    pub fn name_ok(&self) -> bool {
        if self.variable() {
            values::variable_check(&self.name).is_empty()
        } else {
            !self.name.is_empty()
        }
    }
    pub fn encoders(&self) -> Vec<Encoder> {
        spec::encoders(&self.rule, &self.key()).unwrap_or_default()
    }
    /// How the value is edited now.
    pub fn editing_kind(&self) -> Editing {
        self.encoder.map_or(Editing::Nothing, editing)
    }
    /// A new name: the encoder and the value start again, as `onNameChange` cleared them.
    pub fn rename(&mut self, name: String) {
        self.name = name;
        self.choose_encoder(None);
    }
    /// An encoder chosen: the value starts from its default.
    pub fn choose_encoder(&mut self, e: Option<Encoder>) {
        self.encoder = e;
        let start = e.map(|e| values::texts_of(&e.default_value()));
        (self.choice, self.texts) = start.unwrap_or_default();
        self.last = 0;
    }
    /// The value, if the stylesheet can hold it.
    pub fn value(&self) -> Option<Value> {
        let e = self.encoder?;
        values::value_of(e, &self.choice, &self.texts).filter(|v| values::reads_back(e, v))
    }
    pub fn copy_to_all(&mut self) {
        let t = self.texts[self.last.min(3)].clone();
        self.texts = [t.clone(), t.clone(), t.clone(), t];
    }
}

/// Set `s` of `sets`.
pub fn set_mut(sets: &mut PropertySets<Value>, s: usize) -> Option<&mut Vec<(Text, Value)>> {
    match sets {
        PropertySets::Single(props) => (s == 0).then_some(props),
        PropertySets::Multiple(all) => all.get_mut(s),
    }
}
/// Set `s` of `sets`, to read.
pub fn set_ref(sets: &PropertySets<Value>, s: usize) -> Option<&Vec<(Text, Value)>> {
    match sets {
        PropertySets::Single(props) => (s == 0).then_some(props),
        PropertySets::Multiple(all) => all.get(s),
    }
}
/// The set the form edits, in `rules`.
fn form_set<'a>(rules: &'a mut Rules, f: &PropertyForm) -> Option<&'a mut Vec<(Text, Value)>> {
    let (_, sets) = rules.iter_mut().find(|(r, _)| *r == f.rule)?;
    set_mut(sets, f.set)
}
/// `value` as property `key` of `props`: in place of one of that name, else at the end.
fn put(props: &mut Vec<(Text, Value)>, key: Text, value: Value) {
    match props.iter_mut().find(|(n, _)| *n == key) {
        Some(p) => p.1 = value,
        None => props.push((key, value)),
    }
}

impl App {
    /// A tap on the property editor's control.
    pub(super) fn property_tap(&mut self, tap: PropTap) {
        match tap {
            PropTap::Add(r) => self.open_property(r, 0, None),
            PropTap::Edit(r, s, k) => self.open_property(r, s, Some(k)),
            PropTap::All => {
                if let Some(f) = self.theme_editor.prop.as_mut() {
                    f.copy_to_all();
                }
                self.refresh_owned_field();
            }
            PropTap::Delete => self.delete_open_property(),
        }
    }
    /// The editor adding to set `s` of rule `r`, or showing its property `k`.
    fn open_property(&mut self, r: usize, s: usize, k: Option<usize>) {
        let Some((rule, sets)) = self.theme_editor.rules.get_mut(r) else {
            return;
        };
        let rule = rule.clone();
        let Some(props) = set_mut(sets, s) else {
            return;
        };
        let form = match k.map(|k| props.get(k)) {
            None => PropertyForm::adding(rule, s),
            Some(Some((name, value))) => PropertyForm::editing(rule, s, name, value),
            Some(None) => return,
        };
        self.theme_editor.prop = Some(form);
        self.theme_dialog_opened();
        self.navigate(if k.is_some() {
            PROPERTY_EDIT
        } else {
            PROPERTY_ADD
        });
    }
    /// Apply: the value replaces the property's in place, or is added at the end of the set,
    /// unless a name, an encoder or a value is missing or the set has the name already.
    pub(super) fn save_property(&mut self) {
        let Some(f) = self.theme_editor.prop.as_mut() else {
            return;
        };
        let value = f.value().filter(|_| f.name_ok());
        let (key, adding) = (f.key(), f.original.is_none());
        let form = f.clone();
        let e = &mut self.theme_editor;
        let Some(props) = form_set(&mut e.rules, &form) else {
            return;
        };
        let taken = adding && props.iter().any(|(n, _)| *n == key);
        let (Some(value), false) = (value, taken) else {
            if let Some(f) = e.prop.as_mut() {
                (f.error, f.checked) = (if taken { EXISTS } else { "" }, true);
            }
            return;
        };
        put(props, key, value);
        e.prop = None;
        self.theme_changed();
        self.close_editor();
    }
    /// The top bar's Delete: the property goes as the rules' Delete takes it, and the editor
    /// closes.
    fn delete_open_property(&mut self) {
        let Some(f) = self.theme_editor.prop.take() else {
            return;
        };
        let rules = &self.theme_editor.rules;
        let r = rules.iter().position(|(r, _)| *r == f.rule);
        let k = r.and_then(|r| {
            let props = set_ref(&rules[r].1, f.set)?;
            props.iter().position(|(n, _)| f.original == Some(lossy(n)))
        });
        if let (Some(r), Some(k)) = (r, k) {
            self.delete_property(r, f.set, k);
        }
        self.close_editor();
    }
    /// The rule, property and meta editors close.
    pub(super) fn close_theme_forms(&mut self) {
        let e = &mut self.theme_editor;
        (e.form, e.meta_form, e.prop) = (None, None, None);
    }
    /// A field whose text the form changed under it shows the form's text; one gone gives the
    /// editor back to the preview field.
    fn refresh_owned_field(&mut self) {
        let Some(i) = self.field else {
            return;
        };
        match self.line_text(1000 + i as i32) {
            Some(t) if t.encode_utf16().ne(self.editor.text.iter().copied()) => {
                let focused = self.editor.focused;
                self.editor = crate::settings_ui::fields::with_text(&t);
                self.editor.focused = focused;
                self.effects.push(vec!["restartInput".into()]);
            }
            Some(_) => {}
            None => self.release_field(),
        }
    }
    /// The property row `i` shows, if it is one.
    fn prop_part(&self, i: usize) -> Option<Part> {
        self.theme_editor.prop.as_ref()?;
        match self.rows().get(i)?.kind {
            Kind::ThemeEdit(p @ (Part::PropName | Part::PropEncoder | Part::PropValue)) => Some(p),
            _ => None,
        }
    }
    /// What row `i`'s list offers: the names, the encoders by id, or
    /// the values.
    pub(in crate::settings_ui) fn property_choices(&self, i: usize) -> Option<Vec<String>> {
        let part = self.prop_part(i)?;
        let f = self.theme_editor.prop.as_ref()?;
        let mut out = vec![String::new()];
        match part {
            Part::PropName if !f.variable() => out.extend(Self::property_names(f)),
            Part::PropEncoder => out.extend(f.encoders().iter().map(|e| e.id().to_string())),
            Part::PropValue => return self.value_choices(f),
            _ => return None,
        }
        Some(out)
    }
    /// The spec's property names for the rule.
    fn property_names(f: &PropertyForm) -> Vec<String> {
        let all = match f.rule {
            Rule::Font(_) => spec::fonts(),
            _ => spec::elements(),
        };
        all.iter().map(|p| p.name.into()).collect()
    }
    /// The value dropdown's entries for the encoder chosen.
    fn value_choices(&self, f: &PropertyForm) -> Option<Vec<String>> {
        let e = &self.theme_editor;
        let owned = |list: &[&str]| list.iter().map(|s| (*s).to_owned()).collect();
        Some(match f.encoder? {
            Encoder::Variable => {
                let defines = e.rules.iter().find(|(r, _)| *r == Rule::Defines);
                let keys = defines.map_or(vec![], |(_, sets)| match sets {
                    PropertySets::Single(props) => props.iter().map(|p| lossy(&p.0)).collect(),
                    PropertySets::Multiple(_) => vec![],
                });
                [vec![String::new()], keys].concat()
            }
            Encoder::DynamicLight | Encoder::DynamicDark => owned(COLOR_NAMES),
            Encoder::CustomFont => {
                let fonts = e.rules.iter().filter_map(|(r, _)| match r {
                    Rule::Font(name) => Some(lossy(name)),
                    _ => None,
                });
                std::iter::once(String::new()).chain(fonts).collect()
            }
            Encoder::Uri => self.workspace_files(),
            other if editing(other) == Editing::Choice => owned(values::keywords(other)),
            _ => return None,
        })
    }
    /// The workspace's fonts, then its images, as the file selector listed them.
    pub fn workspace_files(&self) -> Vec<String> {
        let c = &self.theme_editor.component;
        let fonts = c.fonts.iter().map(|n| values::flex_uri("fonts", n));
        fonts
            .chain(c.images.iter().map(|n| values::flex_uri("images", n)))
            .collect()
    }
    /// A list entry's label: a name as `translatePropertyName` named it, an encoder's name, or a
    /// value; the empty entry is "select".
    pub(in crate::settings_ui) fn property_choice_label(&self, i: usize, choice: &str) -> String {
        let row = self.prop_part(i);
        let encoder = || choice.parse().ok().and_then(Encoder::from_id);
        match (row, choice) {
            (_, "") => self.label("general__select_dropdown_value_placeholder"),
            (Some(Part::PropEncoder), _) => {
                encoder().map_or_else(String::new, |e| self.label(values::encoder_key(e)))
            }
            (Some(Part::PropName), _) => self.property_label(&text(choice)),
            _ => self.encoded_label(choice),
        }
    }
    /// A value as the dropdown shows it: a variable or colour name as `translatePropertyName`
    /// shows it, anything else as written.
    pub fn encoded_label(&self, choice: &str) -> String {
        let f = self.theme_editor.prop.as_ref();
        match f.and_then(|f| f.encoder) {
            Some(Encoder::Variable | Encoder::DynamicLight | Encoder::DynamicDark) => {
                self.property_label(&text(choice))
            }
            _ => choice.to_owned(),
        }
    }
    /// What row `i`'s list starts on; None when it does not open now.
    pub(super) fn property_selected(&self, i: usize) -> Option<String> {
        let part = self.prop_part(i)?;
        let f = self.theme_editor.prop.as_ref()?;
        match part {
            Part::PropName if f.original.is_none() && !f.variable() => Some(f.name.clone()),
            Part::PropEncoder if f.name_ok() => {
                Some(f.encoder.map_or_else(String::new, |e| e.id().to_string()))
            }
            Part::PropValue if self.value_choices(f).is_some_and(|c| !c.is_empty()) => {
                Some(f.choice.clone())
            }
            _ => None,
        }
    }
    /// A list entry tapped on row `i`; false when the row is not the property editor's.
    pub(in crate::settings_ui) fn property_choose(&mut self, i: usize, choice: &str) -> bool {
        let Some(part) = self.prop_part(i) else {
            return false;
        };
        let Some(f) = self.theme_editor.prop.as_mut() else {
            return false;
        };
        match part {
            Part::PropName => f.rename(choice.into()),
            Part::PropEncoder => f.choose_encoder(choice.parse().ok().and_then(Encoder::from_id)),
            _ => f.choice = choice.into(),
        }
        self.refresh_owned_field();
        true
    }
    /// Line field `part`'s text, or None when it is not one now.
    pub(super) fn property_line(&self, part: Part) -> Option<String> {
        let f = self.theme_editor.prop.as_ref()?;
        match part {
            Part::PropName if f.variable() && f.original.is_none() => Some(f.name.clone()),
            Part::PropValue if f.editing_kind() == Editing::Line => Some(f.texts[0].clone()),
            Part::PropSide(k) if f.editing_kind() == Editing::Sides => f.texts.get(k).cloned(),
            _ => None,
        }
    }
    /// Line field `part` changed.
    pub(super) fn property_typed(&mut self, part: Part, value: String) {
        let Some(f) = self.theme_editor.prop.as_mut() else {
            return;
        };
        match part {
            Part::PropName => f.rename(value),
            Part::PropValue => f.texts[0] = value,
            Part::PropSide(k @ 0..=3) => (f.texts[k], f.last) = (value, k),
            _ => {}
        }
    }
}
