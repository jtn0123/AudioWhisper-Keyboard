//! Key/domain models, selector/state/text-key policy, and the text keyboard's geometry: how wide each key wants to be, where each key's touch and visible
//! bounds fall once a keyboard is laid out at a size, and which key a touch lands on.
//!
//! A port of FlorisBoard's `TextKeyboard.layout` and `getKeyForPos`, `TextKey`'s flex factors and
//! the desired key `TextKeyboardLayout` sizes, giving exactly the same `Float`s: every operation
//! is an `f32` one in the Kotlin's order. The keyboard's tests compare the two (see docs/rust.md).

#![forbid(unsafe_code)]

pub mod aim;
pub mod assembly;
pub mod codes;
mod desired;
mod flex;
mod layout;
pub mod model_data;
pub mod models;
pub mod resize;

pub use desired::desired_key;
pub use flex::{Flex, flex, mode};
pub use layout::{Bounds, KeyFlex, key_at, layout};

/// A rectangle as FlorisBoard's `FlorisRect` holds one: its four edges.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Rect {
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    /// No area: `FlorisRect.isEmpty`, which a NaN edge doesn't make empty.
    pub fn is_empty(&self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }

    /// Whether (x, y) is inside: the left and top edges are, the right and bottom aren't.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}
