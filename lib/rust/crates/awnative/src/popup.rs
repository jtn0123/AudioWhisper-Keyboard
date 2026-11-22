//! A key's long-press popup (the `popup` crate), for `PopupNative`, whose generated entry points
//! call these.
//!
//! Rectangles cross as their four floats, left, top, right and bottom. Results come back as:
//!
//! - an extended popup as an `int[]`, which `PopupLayout` reads: whether it is anchored left (1
//!   or 0), its anchor offset, row 0's and row 1's counts, the initial index, its bounds' four
//!   floats as their bits, then each place's popup index and then each place's row; null if
//!   there is no popup;
//! - the key under the finger as a `long`: the index, or `Long.MIN_VALUE` once the finger is off
//!   the popup;
//! - an element's place as a `long`: its row in the high half and its place in the row in the
//!   low, or -1 if there is none;
//! - a set's order as `PopupOrder.Order`'s arrays: the hint, the prioritized keys, the others,
//!   and which list the others are (0 for the keys listed, -1, -2 and -3 for the set's own list
//!   of relevant keys, symbol popups or number popups). A key is 0 for main, 1 and 2 for the
//!   symbol and number hints, then from 3 the relevant keys, the symbol hint's popup keys and the
//!   number hint's, one after the other; the hint is -1 if none;
//! - a merge as an `int[]` of steps: the index of a key to add (the number of relevant keys for
//!   the merged set's main), or -1 where its main becomes the set's.

use jni_sys::jobjectArray;
use popup::grid::{self, Bounds, Grid};
use popup::order::{self, Contents, HintConfig, HintMode, Key, Other, Step};

use crate::boundary::args::{PopupNativeExtend, PopupNativeOrder, PopupNativePointer};
use crate::jni::{Env, Result, illegal};

/// What `pointer` gives once the finger is off the popup.
const OFF_THE_POPUP: i64 = i64::MIN;

/// A count from Kotlin, which can't be negative.
fn count(count: i32, name: &str) -> Result<usize> {
    usize::try_from(count).map_err(|_| illegal(format!("{name} is negative")))
}

/// The hint mode numbered as `KeyHintMode` orders them.
fn mode(number: i32) -> Result<HintMode> {
    match number {
        0 => Ok(HintMode::Disabled),
        1 => Ok(HintMode::HintPriority),
        2 => Ok(HintMode::AccentPriority),
        3 => Ok(HintMode::SmartPriority),
        _ => Err(illegal("hint mode")),
    }
}

/// A rectangle from its four sides.
fn bounds([left, top, right, bottom]: [f32; 4]) -> Bounds {
    Bounds {
        left,
        top,
        right,
        bottom,
    }
}

/// The extended popup of `count` keys, `prioritized` of them prioritized, for the key at `key_*`
/// whose preview popup is `base_*` (`popup::grid::extend`).
pub fn extend(args: PopupNativeExtend) -> Option<Vec<i32>> {
    let key = bounds([args.key_left, args.key_top, args.key_right, args.key_bottom]);
    let base = bounds([
        args.base_left,
        args.base_top,
        args.base_right,
        args.base_bottom,
    ]);
    let ext = grid::extend(args.count, args.prioritized, key, base, args.keyboard_width)?;
    let g = ext.grid;
    let b = ext.bounds;
    let mut packed = vec![
        i32::from(g.anchor_left),
        g.anchor_offset,
        g.row0_count,
        g.row1_count,
        ext.initial_index,
    ];
    packed.extend([b.left, b.top, b.right, b.bottom].map(|f| f.to_bits() as i32));
    packed.extend(&ext.popup_indices);
    packed.extend((0..args.count).map(|ui_index| ext.row(ui_index)));
    Some(packed)
}

/// Which key of the popup laid out as `anchor_left`, `anchor_offset`, `row0_count` and
/// `row1_count` the finger at (`x`, `y`) is on (`popup::grid::pointer`).
pub fn pointer(args: PopupNativePointer) -> i64 {
    let key = bounds([args.key_left, args.key_top, args.key_right, args.key_bottom]);
    let base = bounds([
        args.base_left,
        args.base_top,
        args.base_right,
        args.base_bottom,
    ]);
    let grid = Grid {
        anchor_left: args.anchor_left,
        anchor_offset: args.anchor_offset,
        row0_count: args.row0_count,
        row1_count: args.row1_count,
    };
    grid::pointer(key, base, grid, args.x, args.y).map_or(OFF_THE_POPUP, i64::from)
}

/// Where element `index` is in rows of these `sizes` (`popup::grid::element_at`).
pub fn element_at(sizes: &[i32], index: i32) -> i64 {
    grid::element_at(sizes, index)
        .map_or(-1, |(row, at)| ((row as i64) << 32) | i64::from(at as u32))
}

/// A key's number, as the module's doc says.
fn key_number(key: Key, contents: &Contents) -> i32 {
    let r = contents.relevant;
    let s = contents.symbol_popups;
    let number = match key {
        Key::Main => 0,
        Key::SymbolHint => 1,
        Key::NumberHint => 2,
        Key::Relevant(at) => 3 + at,
        Key::SymbolPopup(at) => 3 + r + at,
        Key::NumberPopup(at) => 3 + r + s + at,
    };
    number as i32
}

/// The order of a set's keys for the hint settings (`popup::order::order`), as `Order` reads it.
pub fn order(env: &Env, args: PopupNativeOrder) -> Result<jobjectArray> {
    let contents = Contents {
        main: args.main,
        relevant: count(args.relevant, "relevant")?,
        symbol_hint: args.symbol_hint,
        number_hint: args.number_hint,
        symbol_popups: count(args.symbol_popups, "symbolPopups")?,
        number_popups: count(args.number_popups, "numberPopups")?,
    };
    let config = HintConfig {
        symbol: mode(args.symbol_mode)?,
        number: mode(args.number_mode)?,
        merge_hint_popups: args.merge_hint_popups,
    };
    let order = order::order(&contents, config);
    let number = |key: &Key| key_number(*key, &contents);
    let (own_list, others): (i32, &[Key]) = match &order.other {
        Other::Relevant => (-1, &[]),
        Other::SymbolPopups => (-2, &[]),
        Other::NumberPopups => (-3, &[]),
        Other::Keys(keys) => (0, keys),
    };
    let hint = order.hint.as_ref().map_or(-1, number);
    let prioritized: Vec<i32> = order.prioritized.iter().map(number).collect();
    let others: Vec<i32> = others.iter().map(number).collect();
    env.make_objects(&[
        &|| env.make(&[hint]),
        &|| env.make(&prioritized),
        &|| env.make(&others),
        &|| env.make(&[own_list]),
    ])
}

/// Where a merged set's keys go (`popup::order::merge`).
pub fn merge(computed: &[u8], main_computed: bool, use_main: bool, has_main: bool) -> Vec<i32> {
    let computed: Vec<bool> = computed.iter().map(|&c| c != 0).collect();
    let steps = order::merge(&computed, main_computed, use_main, has_main);
    steps
        .iter()
        .map(|step| match step {
            Step::Add(at) => *at as i32,
            Step::SetMain => -1,
        })
        .collect()
}
