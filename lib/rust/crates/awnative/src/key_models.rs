//! Key-domain functions for `KeyModels`, whose generated entry points call these or keys::models
//! directly. Logic remains Java-free in keys; state values cross without native handles.

use keys::models;

use crate::jni::{Result, illegal};

/// The code of every key, as keys::codes lists them.
pub fn codes() -> Vec<i32> {
    keys::codes::VALUES.to_vec()
}

pub fn read(raw: i64, field: i32) -> Result<i32> {
    models::read(raw as u64, field as usize).ok_or_else(|| illegal("unknown state field"))
}

pub fn write(raw: i64, field: i32, value: i32) -> Result<i64> {
    models::write(raw as u64, field as usize, value)
        .map(|v| v as i64)
        .ok_or_else(|| illegal("unknown state field"))
}

pub fn select(kind: i32, raw: i64, present: i32) -> i32 {
    models::select(kind, raw as u64, present as u32)
}

pub fn currency_defaults() -> Vec<i32> {
    models::CURRENCY_DEFAULTS.to_vec()
}

/// Each predefined key as `code,kind,label`.
pub fn predefined() -> Vec<Vec<u16>> {
    keys::model_data::PREDEFINED
        .iter()
        .map(|key| {
            format!("{},{},{}", key.code, key.kind, key.label)
                .encode_utf16()
                .collect()
        })
        .collect()
}

pub fn popup_group(group: i32) -> Option<Vec<u16>> {
    models::popup_group(group).map(|s| s.encode_utf16().collect())
}

pub fn popup_source(variation: i32, present: i32) -> i32 {
    models::popup_source(variation, present as u32)
}

pub fn phone_hint(code: i32) -> Option<Vec<u16>> {
    models::phone_hint(code).map(|s| s.encode_utf16().collect())
}
