//! The declared classes, one file each, named after the awnative module that answers them.

mod shift;

use crate::Class;

/// Every generated class, in the order their entry points are written.
pub const CLASSES: &[Class] = &[shift::SHIFT_NATIVE];
