//! The declared classes, one file each, named after the awnative module that answers them, and
//! the Gradle module each of crates/wire's groups is generated in.

mod clipboard;
mod dictation;
mod layout;
mod packets;
mod rules;
mod screen;
mod shift;

use crate::{Api, Class, Module, Type, Wire};

/// Every generated class, in the order their entry points are written.
pub const CLASSES: &[Class] = &[
    shift::SHIFT_NATIVE,
    rules::IME_WINDOW_RULES,
    rules::GLIDE_TRAIL_RULES,
    rules::RENDER_CONTENT_RULES,
    rules::SUBTYPE_RULES,
    rules::MEDIA_RULES,
    rules::IME_CHROME_RULES,
    packets::SNYGG_GRAMMAR,
    packets::SNYGG_COMPILER,
    packets::SNYGG_VALUES,
    packets::UTILITY_NATIVE,
    packets::BACKUP_NATIVE,
    packets::SETTINGS_NATIVE,
    packets::EXTENSION_POLICY_NATIVE,
    layout::KEY_MODELS,
    layout::LAYOUT_RULES,
    layout::PANEL_RULES,
    clipboard::CLIPBOARD_RULES,
    screen::SCREEN_NATIVE,
    dictation::DICTATION_NATIVE,
];

/// Every generated api over those classes.
pub const APIS: &[Api] = &[
    shift::SHIFT_RULES,
    screen::SCREEN_WALK_RULES,
    dictation::DICTATION_RULES,
    dictation::MIC_KEY_STATE,
];

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
