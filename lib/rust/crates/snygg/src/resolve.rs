//! Compilation and cascade shared by Compose and the future renderer. Generic values let
//! JNI keep opaque Compose objects losslessly; the default is the Java-free `Value`.
use crate::{
    Text,
    grammar::Rule,
    value::{Encoder, Unit, Value},
};

pub type Properties<V = Value> = Vec<(Text, V)>;
pub type QueryAttributes = Vec<(Text, Text)>;

#[derive(Clone, Debug, PartialEq)]
pub enum PropertySets<V = Value> {
    Single(Properties<V>),
    Multiple(Vec<Properties<V>>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Stylesheet<V = Value> {
    pub rules: Vec<(Rule, PropertySets<V>)>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledTheme<V = Value> {
    pub style: Vec<(Rule, Properties<V>)>,
    pub fonts: Vec<(Text, Vec<Properties<V>>)>,
}
#[derive(Clone, Debug)]
pub struct Query<'a> {
    pub element: &'a [u16],
    pub attributes: &'a [(Text, Text)],
    pub selector: usize,
}

/// Classification only. Platform conversion belongs at the caller's boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind<'a> {
    Other,
    Undefined,
    Inherit,
    Variable(&'a [u16]),
    DynamicColor(bool, &'a [u16]),
}
pub trait StyleValue: Clone {
    fn kind(&self) -> Kind<'_>;
    fn undefined() -> Self;
    fn scale(&self, multiplier: f32) -> Self;
}
impl StyleValue for Value {
    fn kind(&self) -> Kind<'_> {
        match self {
            Self::Keyword(Encoder::Undefined) => Kind::Undefined,
            Self::Keyword(Encoder::Inherit) => Kind::Inherit,
            Self::Variable(k) => Kind::Variable(k),
            Self::DynamicColor { dark, name } => Kind::DynamicColor(*dark, name),
            _ => Kind::Other,
        }
    }
    fn undefined() -> Self {
        Self::Keyword(Encoder::Undefined)
    }
    fn scale(&self, multiplier: f32) -> Self {
        match self {
            Self::Dimension(Unit::Sp, n) => Self::Dimension(Unit::Sp, n * multiplier),
            _ => self.clone(),
        }
    }
}

pub fn get<'a, V>(properties: &'a [(Text, V)], key: &[u16]) -> Option<&'a V> {
    properties.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}
/// Replacement keeps the first insertion slot, including after an explicit inherit.
pub fn set<V>(properties: &mut Properties<V>, key: Text, value: Option<V>) -> Option<V> {
    match (properties.iter().position(|(k, _)| *k == key), value) {
        (Some(i), Some(v)) => Some(std::mem::replace(&mut properties[i].1, v)),
        (Some(i), None) => Some(properties.remove(i).1),
        (None, Some(v)) => {
            properties.push((key, v));
            None
        }
        (None, None) => None,
    }
}
pub fn inherit<V: StyleValue>(out: &mut Properties<V>, parent: &Properties<V>) {
    for key in crate::spec::inherited() {
        if get(out, key).is_none()
            && let Some(value) = get(parent, key)
        {
            set(out, key.clone(), Some(value.clone()));
        }
    }
}
pub fn apply<V: StyleValue>(
    out: &mut Properties<V>,
    style: &Properties<V>,
    parent: &Properties<V>,
    multiplier: f32,
) {
    for (key, value) in style {
        let value = match value.kind() {
            Kind::Undefined => None,
            Kind::Inherit => get(parent, key).cloned(),
            _ => Some(value.scale(multiplier)),
        };
        set(out, key.clone(), value);
    }
    inherit(out, parent);
}
pub fn matches(rule: &Rule, query: &Query<'_>) -> bool {
    let Rule::Element {
        name,
        attributes,
        selector,
    } = rule
    else {
        return false;
    };
    name == query.element
        && (*selector == 0 || *selector == query.selector)
        && attributes
            .0
            .iter()
            .all(|(key, values)| get(query.attributes, key).is_some_and(|v| values.contains(v)))
}
impl<V: StyleValue> CompiledTheme<V> {
    /// Annotation shape failures mirror Kotlin's `check`, before variable substitution.
    pub fn compile(sheet: &Stylesheet<V>) -> Result<Self, &'static str> {
        let mut variables = Properties::new();
        let mut style = vec![];
        let mut fonts = vec![];
        for (rule, sets) in &sheet.rules {
            match (rule, sets) {
                (Rule::Defines, PropertySets::Single(p)) => variables = p.clone(),
                (Rule::Font(name), PropertySets::Multiple(p)) => {
                    fonts.push((name.clone(), p.clone()))
                }
                (Rule::Element { .. }, PropertySets::Single(p)) => {
                    style.push((rule.clone(), p.clone()))
                }
                _ => return Err("Check failed."),
            }
        }
        // Group insertion order is observable in Kotlin's internal style map. Sorting
        // stays stable inside each element, including comparator-equivalent rules.
        let mut names = vec![];
        for (r, _) in &style {
            if let Rule::Element { name, .. } = r
                && !names.contains(name)
            {
                names.push(name.clone());
            }
        }
        style.sort_by(|(a, _), (b, _)| {
            let rank = |r: &Rule| match r {
                Rule::Element { name, .. } => names.iter().position(|n| n == name),
                _ => None,
            };
            rank(a).cmp(&rank(b)).then_with(|| a.compare(b).cmp(&0))
        });
        for (_, properties) in &mut style {
            for (_, value) in properties {
                if let Kind::Variable(key) = value.kind() {
                    // One lookup only, deliberately: aliases/cycles remain variable values.
                    *value = get(&variables, key).cloned().unwrap_or_else(V::undefined);
                }
            }
        }
        Ok(Self { style, fonts })
    }
    /// Dynamic colours remain named values for a platform palette to materialize.
    /// An absent element returns inherited values without materializing dynamic colours,
    /// matching SnyggTheme's early return. Use `resolve_with_palette` for final colours.
    pub fn resolve(
        &self,
        query: &Query<'_>,
        parent: &Properties<V>,
        multiplier: f32,
    ) -> Properties<V> {
        let mut out = vec![];
        inherit(&mut out, parent);
        for (rule, properties) in &self.style {
            if matches(rule, query) {
                apply(&mut out, properties, parent, multiplier);
            }
        }
        out
    }
    pub fn resolve_with_palette(
        &self,
        query: &Query<'_>,
        parent: &Properties<V>,
        multiplier: f32,
        light: &Properties<V>,
        dark: &Properties<V>,
    ) -> Properties<V> {
        let mut out = self.resolve(query, parent, multiplier);
        if self
            .style
            .iter()
            .any(|(r, _)| matches!(r, Rule::Element { name, .. } if name == query.element))
        {
            for (_, v) in &mut out {
                if let Kind::DynamicColor(is_dark, name) = v.kind()
                    && let Some(color) = get(if is_dark { dark } else { light }, name)
                {
                    *v = color.clone();
                }
            }
        }
        out
    }
}
