//! The order of a key's popup keys, as `MutablePopupSet` had it: which hint the key shows,
//! which keys are prioritized (drawn nearest the finger) and in what order, for the user's hint
//! settings ([`order`]); and where the keys of a popup set merged into it go ([`merge`]).

/// How a hint (the symbol or the number on a key's corner) is offered, as `KeyHintMode`
/// orders them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HintMode {
    Disabled,
    HintPriority,
    AccentPriority,
    SmartPriority,
}

/// The user's hint settings, as `KeyHintConfiguration`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HintConfig {
    pub symbol: HintMode,
    pub number: HintMode,
    /// Whether a hint's own popup keys join the key's.
    pub merge_hint_popups: bool,
}

/// What a popup set holds: whether it has a main key and each hint, and how many relevant keys
/// and popup keys of each hint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Contents {
    pub main: bool,
    pub relevant: usize,
    pub symbol_hint: bool,
    pub number_hint: bool,
    pub symbol_popups: usize,
    pub number_popups: usize,
}

/// One of a popup set's keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Main,
    SymbolHint,
    NumberHint,
    Relevant(usize),
    SymbolPopup(usize),
    NumberPopup(usize),
}

/// The keys that aren't prioritized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Other {
    /// The set's relevant keys: Kotlin hands over its own list, so keys merged into the set
    /// later show in it too.
    Relevant,
    /// The symbol hint's popup keys, the set's own list likewise.
    SymbolPopups,
    /// The number hint's popup keys, the set's own list likewise.
    NumberPopups,
    Keys(Vec<Key>),
}

/// A popup set's keys in order for one hint configuration, as `PopupKeys`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub hint: Option<Key>,
    pub prioritized: Vec<Key>,
    pub other: Other,
}

/// The keys of a set holding `contents` for the hint settings `config` (`initPopupList`).
pub fn order(contents: &Contents, config: HintConfig) -> Order {
    use HintMode::{AccentPriority, Disabled, HintPriority};
    use Key::{Main, NumberHint, SymbolHint};
    let c = contents;
    let symbol = c.symbol_hint && config.symbol != Disabled;
    let number = c.number_hint && config.number != Disabled;
    let popups = |symbols: bool, numbers: bool| {
        let mut keys = Vec::new();
        if config.merge_hint_popups {
            if symbols {
                keys.extend((0..c.symbol_popups).map(Key::SymbolPopup));
            }
            if numbers {
                keys.extend((0..c.number_popups).map(Key::NumberPopup));
            }
        }
        keys
    };
    // A hint's popup keys alone, which Kotlin hands over as its own list when they are merged.
    let own = |list: Other| {
        if config.merge_hint_popups {
            list
        } else {
            Other::Keys(Vec::new())
        }
    };
    if symbol && number {
        let hint_popups = popups(true, true);
        let keys = |order: &[Key]| keyed(c, SymbolHint, order, &hint_popups);
        match (config.symbol, config.number) {
            // when both hints are present in accent priority, always have a non-hint key
            // first if possible
            (AccentPriority, AccentPriority) => {
                let bare = Other::Keys(hint_popups.clone());
                accent(c, &[SymbolHint, NumberHint], &hint_popups, bare)
            }
            // hint priority of the number hint wins and overrules accent priority of the symbol
            (AccentPriority, HintPriority) => keys(&[NumberHint, Main, SymbolHint]),
            // smart priority of the number hint: main wins if it exists
            (AccentPriority, _) => keys(&[Main, NumberHint, SymbolHint]),
            // when both hints are present in hint priority, the symbol hint wins
            (HintPriority, HintPriority) => keys(&[SymbolHint, NumberHint, Main]),
            // hint priority of the symbol hint overrules the number hint's
            (HintPriority, _) => keys(&[SymbolHint, Main, NumberHint]),
            // smart priority of the symbol hint overrules accent priority of the number hint
            (_, AccentPriority) => keys(&[Main, SymbolHint, NumberHint]),
            // hint priority of the number hint wins
            (_, HintPriority) => keys(&[NumberHint, Main, SymbolHint]),
            // when both hints are in smart priority, always have main first if possible
            _ => keys(&[Main, SymbolHint, NumberHint]),
        }
    } else if symbol {
        let own = own(Other::SymbolPopups);
        one_hint(c, SymbolHint, config.symbol, &popups(true, false), own)
    } else if number {
        let own = own(Other::NumberPopups);
        one_hint(c, NumberHint, config.number, &popups(false, true), own)
    } else {
        // if no hints shall be shown, use main first if possible
        Order {
            hint: None,
            prioritized: present(c, &[Main]),
            other: Other::Relevant,
        }
    }
}

/// The keys of a set showing just one `hint`, offered in `mode`.
fn one_hint(c: &Contents, hint: Key, mode: HintMode, hint_popups: &[Key], own: Other) -> Order {
    match mode {
        // in accent priority, always show a non-hint key first if possible
        HintMode::AccentPriority => accent(c, &[hint], hint_popups, own),
        // in hint priority, always show the hint first
        HintMode::HintPriority => keyed(c, hint, &[hint, Key::Main], hint_popups),
        // in smart priority, show main first if possible
        _ => keyed(c, hint, &[Key::Main, hint], hint_popups),
    }
}

/// Accent priority: main before the `hints` if there is one, else the first relevant key, else
/// the hints alone, with the hints' popup keys as they are (`bare`).
fn accent(c: &Contents, hints: &[Key], hint_popups: &[Key], bare: Other) -> Order {
    let (first, rest) = if c.main {
        (Key::Main, 0)
    } else if c.relevant > 0 {
        (Key::Relevant(0), 1)
    } else {
        return Order {
            hint: Some(hints[0]),
            prioritized: hints.to_vec(),
            other: bare,
        };
    };
    Order {
        hint: Some(hints[0]),
        prioritized: [first].into_iter().chain(hints.iter().copied()).collect(),
        other: Other::Keys(
            (rest..c.relevant)
                .map(Key::Relevant)
                .chain(hint_popups.iter().copied())
                .collect(),
        ),
    }
}

/// `hint`, the keys of `order` the set has, and its relevant keys before the hints' popups.
fn keyed(c: &Contents, hint: Key, order: &[Key], hint_popups: &[Key]) -> Order {
    Order {
        hint: Some(hint),
        prioritized: present(c, order),
        other: Other::Keys(
            (0..c.relevant)
                .map(Key::Relevant)
                .chain(hint_popups.iter().copied())
                .collect(),
        ),
    }
}

/// The keys of `keys` the set has, as Kotlin's `listOfNotNull`.
fn present(c: &Contents, keys: &[Key]) -> Vec<Key> {
    let has = |key: &&Key| match key {
        Key::Main => c.main,
        Key::SymbolHint => c.symbol_hint,
        Key::NumberHint => c.number_hint,
        _ => true,
    };
    keys.iter().filter(has).copied().collect()
}

/// Where one key of a popup set merged into another goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Added to the end of the list merged into: the relevant key with this index, or, at the
    /// number of relevant keys, the merged set's main key.
    Add(usize),
    /// The merged set's main key becomes the set's main key.
    SetMain,
}

/// Where the keys of a popup set go when it is merged into a list (`mergeInternal`): its
/// relevant keys that `computed`, in order, then its main key if it has one that `main_computed`.
/// That becomes the set's main key if this merge may set it (`use_main`) and the set `has_main`
/// not, else it is added after the others.
pub fn merge(computed: &[bool], main_computed: bool, use_main: bool, has_main: bool) -> Vec<Step> {
    let mut steps: Vec<Step> = (0..computed.len())
        .filter(|&at| computed[at])
        .map(Step::Add)
        .collect();
    if main_computed {
        steps.push(if use_main && !has_main {
            Step::SetMain
        } else {
            Step::Add(computed.len())
        });
    }
    steps
}

#[cfg(test)]
mod tests;
