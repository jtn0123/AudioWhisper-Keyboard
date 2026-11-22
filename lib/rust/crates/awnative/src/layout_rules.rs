//! Layout JNI transports primitive metadata and indices; keys never cross into Rust.
use crate::jni::{Result, illegal};
use keys::assembly;

pub fn plan(mode: i32, number_row: bool) -> Result<Vec<i32>> {
    Ok(assembly::plan(mode, number_row).to_vec())
}

pub fn merge_rows(
    main: &[i32],
    modifier: &[i32],
    extension: &[i32],
    codes: &[i32],
) -> Result<Vec<i32>> {
    let rows = assembly::merge(main, modifier, extension, codes).map_err(illegal)?;
    Ok(rows.to_vec())
}

pub fn hints(
    mode: i32,
    number_hints: bool,
    main: &[i32],
    symbols: &[i32],
    types: &[i32],
) -> Result<Vec<i32>> {
    let hints = assembly::hints(mode, number_hints, main, symbols, types).map_err(illegal)?;
    Ok(hints.to_vec())
}

pub fn invalidated(keys: &[i64], mode: i32, subtype_hash: i32, filter: i32) -> Result<Vec<i64>> {
    if !(0..=3).contains(&filter) {
        return Err(illegal("unknown cache filter"));
    }
    let removed = assembly::invalidated(
        keys,
        (filter & 1 != 0).then_some(mode),
        (filter & 2 != 0).then_some(subtype_hash),
    );
    Ok(removed.to_vec())
}
