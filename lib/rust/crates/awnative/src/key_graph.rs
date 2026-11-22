//! One keyboard's key graph (the `keygraph` crate), for `KeyGraphNative`, whose generated entry
//! points call these. JNI transport only: rows in, rows out, no retained JVM references.
use std::sync::Mutex;

use keygraph::{Graph, calls};

use crate::handle::{into_handle, locked};
use crate::jni::{Result, illegal};

/// The keyboard a `KeyGraphBuild` row describes.
pub fn build(fields: &[Vec<u16>]) -> Result<i64> {
    let graph = calls::build(fields).map_err(|error| illegal(error.0))?;
    Ok(into_handle(Mutex::new(graph)))
}

/// The keyboard shown before any is built.
pub fn placeholder() -> i64 {
    into_handle(Mutex::new(Graph::placeholder()))
}

pub fn free(handle: i64) -> Result<()> {
    // SAFETY: the wrapper frees exactly once, when nothing can reach it.
    unsafe { crate::handle::free::<Mutex<Graph>>(handle) };
    Ok(())
}

/// `operation`'s answer to `fields` (`wire::key_graph`).
pub fn call(handle: i64, operation: &[u16], fields: &[Vec<u16>]) -> Result<Vec<Vec<u16>>> {
    // SAFETY: the wrapper keeps the handle live while it calls.
    let mut graph = unsafe { locked::<Graph>(handle, "key graph")? };
    calls::call(&mut graph, operation, fields).map_err(|error| illegal(error.0))
}

/// The keys laid out for `numbers` (`calls::layout`).
pub fn layout(handle: i64, numbers: &[f32]) -> Result<Vec<f32>> {
    // SAFETY: the wrapper keeps the handle live while it calls.
    let mut graph = unsafe { locked::<Graph>(handle, "key graph")? };
    calls::layout(&mut graph, numbers).map_err(|error| illegal(error.0))
}
