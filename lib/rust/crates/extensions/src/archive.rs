//! The historical ZIP policy. Sizes above the per-entry cap truncate/create an empty file;
//! duplicate entries and missing parents are left to the same ZIP/file I/O as before.
use crate::Text;

#[derive(Clone, Debug)]
pub struct Entry<'a> {
    pub name: &'a [u16],
    pub canonical_path: &'a [u16],
    pub size: i64,
    pub directory: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Skip,
    Directory,
    Copy,
    Empty,
}
impl Action {
    pub fn name(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::Directory => "directory",
            Self::Copy => "copy",
            Self::Empty => "empty",
        }
    }
}

pub fn names(entries: &[Text]) -> Vec<bool> {
    entries.iter().map(|name| name.len() <= 255).collect()
}

pub fn plan(root: &[u16], entries: &[Entry<'_>]) -> Vec<Action> {
    let mut prefix = root.to_vec();
    prefix.push(47);
    entries
        .iter()
        .map(|entry| {
            if entry.name.len() > 255
                || entry.canonical_path.len() > 1023
                || !entry.canonical_path.starts_with(&prefix)
            {
                Action::Skip
            } else if entry.directory {
                Action::Directory
            } else if entry.size > 100_000_000 {
                Action::Empty
            } else {
                Action::Copy
            }
        })
        .collect()
}
