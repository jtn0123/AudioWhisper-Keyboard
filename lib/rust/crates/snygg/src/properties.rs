//! Ordered decoding with deferred JSON errors, preserving source-order error precedence.
use crate::{
    Text,
    value::{Encoder, Value},
};
#[derive(Clone, Debug)]
pub struct PropertyInput {
    pub name: Text,
    pub encoders: Option<Vec<Encoder>>,
    pub input: Result<Text, ()>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyError {
    InvalidProperty(usize),
    InvalidValue(usize),
    Json(usize),
    Missing(Text),
}
#[derive(Clone, Debug, Default)]
pub struct Configuration {
    pub ignore_properties: bool,
    pub ignore_values: bool,
}
pub fn decode(
    entries: &[PropertyInput],
    required: &[Text],
    config: &Configuration,
) -> Result<Vec<(usize, Value)>, PropertyError> {
    let mut values = vec![];
    for (index, entry) in entries.iter().enumerate() {
        let Some(encoders) = &entry.encoders else {
            if config.ignore_properties {
                continue;
            }
            return Err(PropertyError::InvalidProperty(index));
        };
        let input = entry
            .input
            .as_ref()
            .map_err(|_| PropertyError::Json(index))?;
        match Encoder::select(encoders, input) {
            Some(value) => values.push((index, value)),
            None if config.ignore_values => {}
            None => return Err(PropertyError::InvalidValue(index)),
        }
    }
    if let Some(key) = required
        .iter()
        .find(|key| !values.iter().any(|(i, _)| entries[*i].name == **key))
    {
        return Err(PropertyError::Missing(key.clone()));
    }
    Ok(values)
}
pub fn missing<'a>(names: &[Text], required: &'a [Text]) -> Option<&'a Text> {
    required.iter().find(|key| !names.contains(key))
}
