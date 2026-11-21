//! The declared classes, one file each, named after the awnative module that answers them, and
//! the Gradle module each of crates/wire's groups is generated in.

mod shift;

use crate::{Class, Module, Wire};

/// Every generated class, in the order their entry points are written.
pub const CLASSES: &[Class] = &[shift::SHIFT_NATIVE];

/// Every group crates/wire declares, and the Kotlin module that calls with it.
pub const WIRE: &[Wire] = &[
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::flag::GROUPS,
    },
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::render::GROUPS,
    },
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::input::GROUPS,
    },
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::extensions::GROUPS,
    },
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::settings::GROUPS,
    },
    // lib/kotlin can't see lib/rust, so it has its own copy for its own calls.
    Wire {
        module: Module::Kotlin,
        internal: true,
        groups: wire::flag::GROUPS,
    },
];
