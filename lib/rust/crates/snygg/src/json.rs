//! Standalone input for a Rust renderer. Kotlin retains its kotlinx codec and exact
//! diagnostics; this strict entry point accepts the same valid bundled v2 documents.
use crate::{
    Text, document, grammar, properties,
    resolve::{CompiledTheme, Properties, PropertySets, Stylesheet},
    spec, text,
    value::Value,
};
use serde_json::Value as Json;

#[derive(Clone, Debug, PartialEq)]
pub struct JsonError(pub String);
impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for JsonError {}
fn error(message: &str) -> JsonError {
    JsonError(message.to_owned())
}
fn message(e: crate::Error) -> JsonError {
    JsonError(String::from_utf16_lossy(&e.message()))
}
fn object(value: &Json) -> Result<&serde_json::Map<String, Json>, JsonError> {
    value
        .as_object()
        .ok_or_else(|| error("expected a property object"))
}
fn decode(rule: &grammar::Rule, input: &Json) -> Result<Properties, JsonError> {
    let entries = object(input)?
        .iter()
        .map(|(name, value)| {
            let name = text(name);
            properties::PropertyInput {
                encoders: spec::encoders(rule, &name),
                name,
                input: value.as_str().map(text).ok_or(()),
            }
        })
        .collect::<Vec<_>>();
    let required: Vec<Text> = if matches!(rule, grammar::Rule::Font(_)) {
        vec![text("src")]
    } else {
        vec![]
    };
    let decoded = properties::decode(&entries, &required, &properties::Configuration::default())
        .map_err(|e| JsonError(format!("property decoding failed: {e:?}")))?;
    Ok(decoded
        .into_iter()
        .map(|(i, v)| (entries[i].name.clone(), v))
        .collect())
}
impl Stylesheet<Value> {
    pub fn from_json(input: &str) -> Result<Self, JsonError> {
        let json: Json = serde_json::from_str(input).map_err(|e| JsonError(e.to_string()))?;
        let obj = object(&json)?;
        let config = document::Configuration::default();
        let schema = obj.get("$schema").and_then(Json::as_str).map(text);
        document::schema(schema.as_deref(), config).map_err(message)?;
        let keys: Vec<_> = obj.keys().map(|s| text(s)).collect();
        let values: Vec<_> = obj.values().collect();
        let mut rules: Vec<(grammar::Rule, PropertySets)> = vec![];
        for entry in document::plan(&keys, config) {
            let rule = entry.rule.map_err(message)?;
            let sets = if matches!(rule, grammar::Rule::Font(_)) {
                let input = values[entry.source]
                    .as_array()
                    .ok_or_else(|| error("expected font sets"))?;
                PropertySets::Multiple(
                    input
                        .iter()
                        .map(|v| decode(&rule, v))
                        .collect::<Result<_, _>>()?,
                )
            } else {
                PropertySets::Single(decode(&rule, values[entry.source])?)
            };
            if entry.slot == rules.len() {
                rules.push((rule, sets));
            } else {
                rules[entry.slot].1 = sets;
            }
        }
        Ok(Self { rules })
    }
}
impl CompiledTheme<Value> {
    pub fn from_json(input: &str) -> Result<Self, JsonError> {
        Self::compile(&Stylesheet::from_json(input)?).map_err(error)
    }
}
