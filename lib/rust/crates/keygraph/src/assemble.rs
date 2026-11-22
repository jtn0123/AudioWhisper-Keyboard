//! A keyboard's rows from its layout files, as `LayoutManager.mergeLayouts` merged them.

use std::collections::HashMap;

use keys::assembly;

use crate::graph::Files;
use crate::node::{Arena, Mapping, Node};
use crate::parse;

/// One keyboard's files, each parsed once however often the keyboard reads it, as
/// `LayoutManager`'s caches held one object per layout and per popup mapping: a number row the
/// letters and their symbol hints both read, or a subtype mapping that is the default one, gives
/// its keys one popup set each, not two.
#[derive(Default)]
pub struct Reader<'a> {
    pub arena: Arena,
    layouts: HashMap<&'a str, Option<Vec<Vec<Node>>>>,
    mappings: HashMap<&'a str, Option<Mapping>>,
}

impl<'a> Reader<'a> {
    /// A layout file's rows, `None` if it doesn't parse.
    pub fn layout(&mut self, text: &'a str) -> Option<Vec<Vec<Node>>> {
        if let Some(rows) = self.layouts.get(text) {
            return rows.clone();
        }
        let rows = parse::layout(text, &mut self.arena);
        self.layouts.insert(text, rows.clone());
        rows
    }

    /// A popup mapping file's mapping, `None` if it doesn't parse.
    pub fn mapping(&mut self, text: &'a str) -> Option<Mapping> {
        if let Some(mapping) = self.mappings.get(text) {
            return mapping.clone();
        }
        let mapping = parse::mapping(text, &mut self.arena);
        self.mappings.insert(text, mapping.clone());
        mapping
    }
}

/// A layout's rows by its files: main, then its own modifier or the subtype's, then the
/// extension, merged.
pub fn assemble<'a>(files: &Files<'a>, reader: &mut Reader<'a>) -> Vec<Vec<Node>> {
    let main = files.main.and_then(|text| reader.layout(text));
    let modifier = match (&main, files.own_modifier) {
        (Some(_), Some(own)) => own,
        _ => files.modifier,
    };
    let modifier = modifier.and_then(|text| reader.layout(text));
    let extension = files.extension.and_then(|text| reader.layout(text));
    let mod_codes: Vec<i32> = modifier
        .iter()
        .flatten()
        .flatten()
        .map(|node| match node {
            Node::Text(leaf) => leaf.code,
            _ => i32::MIN,
        })
        .collect();
    let packed = assembly::merge(
        &optional_sizes(&main),
        &optional_sizes(&modifier),
        &optional_sizes(&extension),
        &mod_codes,
    )
    .unwrap_or_default();
    let flat: Vec<&Node> = [&main, &modifier, &extension]
        .into_iter()
        .flatten()
        .flatten()
        .flatten()
        .collect();
    let mut rows = Vec::new();
    let mut at = 0;
    while let Some(&size) = packed.get(at) {
        let indices = packed
            .get(at + 1..at + 1 + size as usize)
            .unwrap_or_default();
        rows.push(
            indices
                .iter()
                .map(|&key| flat[key as usize].clone())
                .collect(),
        );
        at += 1 + size as usize;
    }
    rows
}

fn optional_sizes(rows: &Option<Vec<Vec<Node>>>) -> Vec<i32> {
    rows.as_ref().map_or(vec![-1], |rows| sizes(rows))
}

pub fn sizes(rows: &[Vec<Node>]) -> Vec<i32> {
    rows.iter().map(|row| row.len() as i32).collect()
}
