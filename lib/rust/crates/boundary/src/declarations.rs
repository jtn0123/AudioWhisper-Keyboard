//! The declared classes, one file each, named after the awnative module that answers them, and
//! the Gradle module each of crates/wire's groups is generated in.

mod shift;

use crate::{Api, Class, Module, Type, Wire};

/// Every generated class, in the order their entry points are written.
pub const CLASSES: &[Class] = &[shift::SHIFT_NATIVE];

/// Every generated api over those classes.
pub const APIS: &[Api] = &[shift::SHIFT_RULES];

/// Every generated top-level record and enum, with the declarations file and the module it is in.
pub const TYPES: &[(&str, Module, Type)] = &[];

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
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::backup::GROUPS,
    },
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::editor::GROUPS,
    },
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::nlp::GROUPS,
    },
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::dictation::GROUPS,
    },
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::playback::GROUPS,
    },
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::settings_ui::GROUPS,
    },
    // lib/kotlin can't see lib/rust, so it has its own copy for its own calls.
    Wire {
        module: Module::Kotlin,
        internal: true,
        groups: wire::flag::GROUPS,
    },
    Wire {
        module: Module::Kotlin,
        internal: false,
        groups: wire::utilities::GROUPS,
    },
];
