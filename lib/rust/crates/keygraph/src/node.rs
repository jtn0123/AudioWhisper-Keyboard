//! A key as its layout file describes it, before any state picks or cases it: FlorisBoard's
//! `AbstractKeyData` and its subclasses, `PopupSet` and `PopupMapping`.

use std::collections::HashMap;

/// One key description: a key, or a selector over keys.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// `text_key`, and any `$` FlorisBoard doesn't know, which it reads as one.
    Text(Leaf),
    /// `auto_text_key`: a letter cased for the subtype's locale. The number is its entry in
    /// [`Arena::autos`].
    Auto(Leaf, usize),
    /// `multi_text_key`: several code points. The number is its entry in [`Arena::multis`], which
    /// is also its identity: Kotlin compares these objects by reference.
    Multi(Leaf, usize),
    /// A selector (`keys::models::select`'s kind) and its children in Kotlin's order; `None` for
    /// one the file left out or set to null.
    Select(i32, Vec<Option<Node>>),
}

/// The fields every key carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leaf {
    /// The `KeyType` ordinal.
    pub kind: i32,
    pub code: i32,
    pub label: Vec<u16>,
    pub group: i32,
    /// Its own popup set, an index into [`Arena::popups`].
    pub popup: Option<usize>,
}

impl Leaf {
    /// The key Kotlin's `TextKeyData()` is: a character with code 0 and no label.
    pub fn unspecified() -> Self {
        Leaf {
            kind: 0,
            code: 0,
            label: Vec::new(),
            group: 0,
            popup: None,
        }
    }
}

/// A popup set as a file describes it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Popup {
    pub main: Option<Node>,
    pub relevant: Vec<Node>,
}

/// A popup mapping: for each `KeyVariation` value (0 all, 1 email, 2 normal, 3 password, 4 uri),
/// the popup set of each label, an index into [`Arena::popups`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mapping {
    pub variations: [Option<HashMap<Vec<u16>, usize>>; 5],
}

impl Mapping {
    /// The popup set the mapping gives `label` under `variation`.
    pub fn get(&self, variation: usize, label: &[u16]) -> Option<usize> {
        self.variations[variation].as_ref()?.get(label).copied()
    }
}

/// Everything the files of one keyboard hold that more than one key refers to.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Arena {
    pub popups: Vec<Popup>,
    /// Each `auto_text_key`'s code and label, as written.
    pub autos: Vec<(i32, Vec<u16>)>,
    /// Each `multi_text_key`'s code points.
    pub multis: Vec<Vec<i32>>,
}

impl Arena {
    /// How much it holds, to roll a failed file back to.
    pub fn mark(&self) -> [usize; 3] {
        [self.popups.len(), self.autos.len(), self.multis.len()]
    }

    /// Drops what was added since `mark`.
    pub fn rollback(&mut self, mark: [usize; 3]) {
        self.popups.truncate(mark[0]);
        self.autos.truncate(mark[1]);
        self.multis.truncate(mark[2]);
    }
}

/// Every leaf of `node`, the leaves of a selector's children included.
pub fn leaves<'a>(node: &'a Node, out: &mut Vec<&'a Leaf>) {
    match node {
        Node::Text(leaf) | Node::Auto(leaf, _) | Node::Multi(leaf, _) => out.push(leaf),
        Node::Select(_, children) => {
            for child in children.iter().flatten() {
                leaves(child, out);
            }
        }
    }
}
