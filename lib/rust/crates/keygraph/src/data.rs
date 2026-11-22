//! A key as the keyboard's state computes it, FlorisBoard's `KeyData`, and the popup keys
//! gathered for it, `MutablePopupSet`.

use popup::order::{self, Contents, HintConfig, Other};

/// A computed key: a `TextKeyData`, or the `MultiTextKeyData` of [`Data::multi`].
///
/// Two are equal exactly when Kotlin's objects were: a text key by its fields, its popup set
/// by identity, and a multi-text key by identity alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Data {
    /// The `KeyType` ordinal.
    pub kind: i32,
    pub code: i32,
    pub label: Vec<u16>,
    pub group: i32,
    /// The popup set it came with, an index into the graph's arena.
    pub popup: Option<usize>,
    /// For a multi-text key, its entry in the arena: its code points and its identity.
    pub multi: Option<usize>,
}

impl Data {
    /// `TextKeyData.UNSPECIFIED`: what a hidden key holds.
    pub fn unspecified() -> Self {
        Data {
            kind: 9,
            code: 0,
            label: "unspecified".encode_utf16().collect(),
            group: 0,
            popup: None,
            multi: None,
        }
    }

    /// Whether it is a space, as `KeyData.isSpaceKey`.
    pub fn is_space(&self) -> bool {
        keys::models::is_space(self.kind, self.code)
    }
}

/// Which of a set's lists a merge adds to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Relevant,
    SymbolPopups,
    NumberPopups,
}

/// The popup keys gathered for one key, `MutablePopupSet`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Popups {
    pub main: Option<Data>,
    pub relevant: Vec<Data>,
    pub symbol_hint: Option<Data>,
    pub number_hint: Option<Data>,
    pub symbol_popups: Vec<Data>,
    pub number_popups: Vec<Data>,
}

/// A set's keys in order for one hint configuration, `PopupKeys`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PopupKeys {
    pub hint: Option<Data>,
    pub prioritized: Vec<Data>,
    pub other: Vec<Data>,
}

impl PopupKeys {
    /// Every key, prioritized first, as `PopupKeys` iterates.
    pub fn all(&self) -> impl Iterator<Item = &Data> {
        self.prioritized.iter().chain(&self.other)
    }

    pub fn is_empty(&self) -> bool {
        self.prioritized.is_empty() && self.other.is_empty()
    }
}

impl Popups {
    /// Merges a popup set's computed keys (`relevant`, then `main`, each `None` where it computed
    /// to nothing) into `target`, as `mergeInternal`; only a merge into the relevant keys may set
    /// the main key.
    pub fn merge(&mut self, relevant: Vec<Option<Data>>, main: Option<Data>, target: Target) {
        let computed: Vec<bool> = relevant.iter().map(Option::is_some).collect();
        let use_main = target == Target::Relevant;
        let steps = order::merge(&computed, main.is_some(), use_main, self.main.is_some());
        for step in steps {
            let key = match step {
                order::Step::SetMain => {
                    self.main = main.clone();
                    continue;
                }
                order::Step::Add(at) if at < relevant.len() => relevant[at].clone(),
                order::Step::Add(_) => main.clone(),
            };
            let list = match target {
                Target::Relevant => &mut self.relevant,
                Target::SymbolPopups => &mut self.symbol_popups,
                Target::NumberPopups => &mut self.number_popups,
            };
            list.extend(key);
        }
    }

    /// The keys for the hint settings `config`, as `getPopupKeys`.
    pub fn keys(&self, config: HintConfig) -> PopupKeys {
        let contents = Contents {
            main: self.main.is_some(),
            relevant: self.relevant.len(),
            symbol_hint: self.symbol_hint.is_some(),
            number_hint: self.number_hint.is_some(),
            symbol_popups: self.symbol_popups.len(),
            number_popups: self.number_popups.len(),
        };
        let order = order::order(&contents, config);
        let key = |key: &order::Key| self.key(*key);
        PopupKeys {
            hint: order.hint.as_ref().and_then(key),
            prioritized: order.prioritized.iter().filter_map(key).collect(),
            other: match &order.other {
                Other::Relevant => self.relevant.clone(),
                Other::SymbolPopups => self.symbol_popups.clone(),
                Other::NumberPopups => self.number_popups.clone(),
                Other::Keys(keys) => keys.iter().filter_map(key).collect(),
            },
        }
    }

    fn key(&self, key: order::Key) -> Option<Data> {
        match key {
            order::Key::Main => self.main.clone(),
            order::Key::SymbolHint => self.symbol_hint.clone(),
            order::Key::NumberHint => self.number_hint.clone(),
            order::Key::Relevant(at) => self.relevant.get(at).cloned(),
            order::Key::SymbolPopup(at) => self.symbol_popups.get(at).cloned(),
            order::Key::NumberPopup(at) => self.number_popups.get(at).cloned(),
        }
    }
}
