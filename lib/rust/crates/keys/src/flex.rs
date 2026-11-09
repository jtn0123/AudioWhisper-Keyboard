//! How much each key of a text keyboard wants to shrink, grow and be wide, from `TextKey.compute`.

/// The keyboard modes the factors tell apart, by `KeyboardMode.value`.
pub mod mode {
    pub const CHARACTERS: i32 = 0;
    pub const SYMBOLS: i32 = 2;
    pub const SYMBOLS2: i32 = 3;
    pub const NUMERIC: i32 = 4;
    pub const NUMERIC_ADVANCED: i32 = 5;
    pub const PHONE: i32 = 6;
    pub const PHONE2: i32 = 7;
}

/// The key codes the factors tell apart (FlorisBoard's `KeyCode`).
mod code {
    pub const ENTER: i32 = 10;
    pub const SPACE: i32 = 32;
    pub const COMMA: i32 = 44;
    pub const PERIOD: i32 = 46;
    pub const EQUALS: i32 = 61;
    pub const DELETE: i32 = -7;
    pub const SHIFT: i32 = -11;
    pub const VIEW_CHARACTERS: i32 = -201;
    pub const VIEW_SYMBOLS: i32 = -202;
    pub const VIEW_SYMBOLS2: i32 = -203;
    pub const CJK_SPACE: i32 = 12288;
}

/// A visible key's flex factors: `flayShrink`, `flayGrow` and `flayWidthFactor`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flex {
    pub shrink: f32,
    pub grow: f32,
    pub width_factor: f32,
}

/// The factors of a visible key with this `code` on a keyboard of this `mode`
/// (`KeyboardMode.value`); `numeric` if the key's type is `KeyType.NUMERIC`.
pub fn flex(mode: i32, code: i32, numeric: bool) -> Flex {
    Flex {
        shrink: shrink(mode, code),
        grow: grow(mode, code, numeric),
        width_factor: width_factor(mode, code),
    }
}

fn shrink(mode: i32, code: i32) -> f32 {
    match mode {
        mode::NUMERIC | mode::NUMERIC_ADVANCED | mode::PHONE | mode::PHONE2 => 1.0,
        _ => match code {
            code::SHIFT | code::DELETE => 1.5,
            code::VIEW_CHARACTERS | code::VIEW_SYMBOLS | code::VIEW_SYMBOLS2 | code::ENTER => 0.0,
            _ => 1.0,
        },
    }
}

fn grow(mode: i32, code: i32, numeric: bool) -> f32 {
    match mode {
        mode::NUMERIC | mode::PHONE | mode::PHONE2 => 0.0,
        mode::NUMERIC_ADVANCED => {
            if numeric {
                1.0
            } else {
                0.0
            }
        }
        _ => match code {
            code::SPACE | code::CJK_SPACE => 1.0,
            _ => 0.0,
        },
    }
}

fn width_factor(mode: i32, code: i32) -> f32 {
    match mode {
        mode::NUMERIC | mode::PHONE | mode::PHONE2 => 2.68,
        mode::NUMERIC_ADVANCED => match code {
            code::COMMA | code::PERIOD => 1.00,
            code::VIEW_SYMBOLS | code::EQUALS => 1.26,
            _ => 1.56,
        },
        _ => match code {
            code::SHIFT | code::DELETE => 1.56,
            code::VIEW_CHARACTERS | code::VIEW_SYMBOLS | code::VIEW_SYMBOLS2 | code::ENTER => 1.56,
            _ => 1.00,
        },
    }
}

#[cfg(test)]
mod tests;
