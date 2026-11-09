use super::*;
use HintMode::{AccentPriority, Disabled, HintPriority, SmartPriority};
use Key::{Main, NumberHint, NumberPopup, Relevant, SymbolHint, SymbolPopup};

fn config(symbol: HintMode, number: HintMode, merge: bool) -> HintConfig {
    HintConfig {
        symbol,
        number,
        merge_hint_popups: merge,
    }
}

/// A set with a main key, two relevant keys, both hints and a popup key for each.
const FULL: Contents = Contents {
    main: true,
    relevant: 2,
    symbol_hint: true,
    number_hint: true,
    symbol_popups: 1,
    number_popups: 1,
};

fn prioritized(contents: &Contents, symbol: HintMode, number: HintMode) -> Vec<Key> {
    order(contents, config(symbol, number, false)).prioritized
}

#[test]
fn without_hints_main_comes_first_and_the_relevant_list_is_kept() {
    let order = order(&FULL, config(Disabled, Disabled, true));
    assert_eq!(order.hint, None);
    assert_eq!(order.prioritized, [Main]);
    assert_eq!(order.other, Other::Relevant);
    let no_main = Contents {
        main: false,
        ..FULL
    };
    assert_eq!(prioritized(&no_main, Disabled, Disabled), []);
}

#[test]
fn both_hints_order_as_their_modes_say() {
    let cases = [
        (HintPriority, HintPriority, [SymbolHint, NumberHint, Main]),
        (HintPriority, AccentPriority, [SymbolHint, Main, NumberHint]),
        (HintPriority, SmartPriority, [SymbolHint, Main, NumberHint]),
        (AccentPriority, HintPriority, [NumberHint, Main, SymbolHint]),
        (SmartPriority, HintPriority, [NumberHint, Main, SymbolHint]),
        (
            AccentPriority,
            SmartPriority,
            [Main, NumberHint, SymbolHint],
        ),
        (
            SmartPriority,
            AccentPriority,
            [Main, SymbolHint, NumberHint],
        ),
        (SmartPriority, SmartPriority, [Main, SymbolHint, NumberHint]),
        (
            AccentPriority,
            AccentPriority,
            [Main, SymbolHint, NumberHint],
        ),
    ];
    for (symbol, number, expected) in cases {
        assert_eq!(
            prioritized(&FULL, symbol, number),
            expected,
            "{symbol:?} {number:?}"
        );
    }
}

#[test]
fn accent_priority_puts_a_relevant_key_first_without_main() {
    let no_main = Contents {
        main: false,
        ..FULL
    };
    let both = order(&no_main, config(AccentPriority, AccentPriority, true));
    assert_eq!(both.hint, Some(SymbolHint));
    assert_eq!(both.prioritized, [Relevant(0), SymbolHint, NumberHint]);
    assert_eq!(
        both.other,
        Other::Keys(vec![Relevant(1), SymbolPopup(0), NumberPopup(0)])
    );
    let bare = Contents {
        relevant: 0,
        ..no_main
    };
    let one = order(&bare, config(Disabled, AccentPriority, true));
    assert_eq!(one.hint, Some(NumberHint));
    assert_eq!(one.prioritized, [NumberHint]);
    assert_eq!(one.other, Other::NumberPopups);
    let unmerged = order(&bare, config(AccentPriority, Disabled, false));
    assert_eq!(unmerged.other, Other::Keys(vec![]));
}

#[test]
fn one_hint_without_main_leaves_it_out() {
    let no_main = Contents {
        main: false,
        ..FULL
    };
    assert_eq!(prioritized(&no_main, HintPriority, Disabled), [SymbolHint]);
    assert_eq!(prioritized(&no_main, Disabled, SmartPriority), [NumberHint]);
    assert_eq!(
        prioritized(&no_main, HintPriority, SmartPriority),
        [SymbolHint, NumberHint]
    );
}

#[test]
fn a_missing_or_disabled_hint_is_left_out() {
    let no_number = Contents {
        number_hint: false,
        ..FULL
    };
    let symbol = order(&no_number, config(SmartPriority, HintPriority, true));
    assert_eq!(symbol.prioritized, [Main, SymbolHint]);
    assert_eq!(
        symbol.other,
        Other::Keys(vec![Relevant(0), Relevant(1), SymbolPopup(0)])
    );
    assert_eq!(
        prioritized(&FULL, Disabled, HintPriority),
        [NumberHint, Main]
    );
}

#[test]
fn hint_popups_join_only_when_merged() {
    let order = order(&FULL, config(SmartPriority, SmartPriority, false));
    assert_eq!(order.other, Other::Keys(vec![Relevant(0), Relevant(1)]));
}

#[test]
fn merging_adds_what_computed_then_main() {
    assert_eq!(
        merge(&[true, false, true], true, false, false),
        [Step::Add(0), Step::Add(2), Step::Add(3)]
    );
    assert_eq!(
        merge(&[true], true, true, false),
        [Step::Add(0), Step::SetMain]
    );
    assert_eq!(
        merge(&[true], true, true, true),
        [Step::Add(0), Step::Add(1)]
    );
    assert_eq!(merge(&[false], false, true, false), []);
}
