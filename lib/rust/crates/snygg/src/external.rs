//! Lossless opaque-value transport for external style hosts. Tokens are owned UTF-16
//! identifiers, not pointers or VM references. Classification/scaling stays in the core;
//! a host only converts platform objects to and from these plain records.
use crate::{
    Text,
    resolve::{Kind, StyleValue},
};

#[derive(Clone, Debug)]
pub enum ExternalValue {
    Opaque(Text),
    Undefined,
    Inherit,
    Variable(Text),
    Sp(f32, bool),
    Dynamic(bool, Text),
}
impl StyleValue for ExternalValue {
    fn kind(&self) -> Kind<'_> {
        match self {
            Self::Undefined => Kind::Undefined,
            Self::Inherit => Kind::Inherit,
            Self::Variable(k) => Kind::Variable(k),
            Self::Dynamic(d, k) => Kind::DynamicColor(*d, k),
            _ => Kind::Other,
        }
    }
    fn undefined() -> Self {
        Self::Undefined
    }
    fn scale(&self, multiplier: f32) -> Self {
        match self {
            Self::Sp(n, em) => Self::Sp(n * multiplier, *em),
            _ => self.clone(),
        }
    }
}
