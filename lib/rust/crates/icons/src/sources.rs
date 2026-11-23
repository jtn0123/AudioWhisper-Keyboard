//! `data/sources.tsv`: each icon's name, its viewport's width and height, then each of its paths'
//! nodes, as Compose held them. `IconSourcesTest` (app/src/test) writes and checks the file.

use std::collections::HashMap;
use std::sync::OnceLock;

/// One icon's vector: its viewport and its paths, in the canonical syntax `nodes` reads.
#[derive(Debug)]
pub(crate) struct Source {
    pub name: &'static str,
    pub width: f32,
    pub height: f32,
    pub paths: Vec<&'static str>,
}

const DATA: &str = include_str!("../data/sources.tsv");

fn read(line: &'static str) -> Option<Source> {
    let mut fields = line.split('\t');
    let name = fields.next()?;
    let width = fields.next()?.parse().ok()?;
    let height = fields.next()?.parse().ok()?;
    Some(Source {
        name,
        width,
        height,
        paths: fields.collect(),
    })
}

/// Every icon, in the file's order.
pub(crate) fn all() -> &'static [Source] {
    static ALL: OnceLock<Vec<Source>> = OnceLock::new();
    ALL.get_or_init(|| {
        let lines = DATA
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#'));
        lines.filter_map(read).collect()
    })
}

/// Icon `name`'s place in [`all`].
pub(crate) fn index(name: &str) -> Option<usize> {
    static INDEX: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    let index = INDEX.get_or_init(|| all().iter().enumerate().map(|(i, s)| (s.name, i)).collect());
    index.get(name).copied()
}
