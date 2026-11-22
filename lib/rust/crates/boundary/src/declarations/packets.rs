//! Objects that carry whole messages of UTF-16 rows: an operation code and its input rows in, the
//! answer's rows out. Each calls a crate's `call` or a module's `dispatch`, whose refusal throws
//! `IllegalArgumentException`.

use crate::{CLASS, Class, METHOD, Method, Module, Param, Return};

/// The one method every packet object has.
const fn call(rust: &'static str) -> Method {
    Method {
        name: "call",
        params: &[("operation", Param::Int), ("input", Param::Strings)],
        returns: Return::Strings,
        rust,
        ..METHOD
    }
}

pub const SNYGG_GRAMMAR: Class = Class {
    name: "SnyggGrammar",
    doc: "Stateless UTF-16 transport for the Java-free snygg crate. The Snygg API owns object \
          allocation.",
    module: "snygg",
    file: "packets",
    public: true,
    methods: &[call("dispatch")],
    ..CLASS
};

pub const SNYGG_COMPILER: Class = Class {
    name: "SnyggCompiler",
    doc: "UTF-16/object-token transport to the Java-free compiler and editor.",
    module: "snygg_resolve",
    file: "packets",
    public: true,
    methods: &[call("dispatch")],
    ..CLASS
};

pub const SNYGG_VALUES: Class = Class {
    name: "SnyggValues",
    doc: "Stateless UTF-16 and f32-bit transport for Snygg values and ordered property decoding.",
    module: "snygg_value",
    file: "packets",
    public: true,
    methods: &[call("dispatch")],
    ..CLASS
};

pub const UTILITY_NATIVE: Class = Class {
    name: "UtilityNative",
    doc: "lib/kotlin's utilities in Rust (lib/rust/crates/utilities). [UtilityRules] wraps it.",
    module: "utilities",
    file: "packets",
    kotlin: Module::Kotlin,
    methods: &[call("::utilities::call")],
    ..CLASS
};

pub const BACKUP_NATIVE: Class = Class {
    name: "BackupNative",
    doc: "Backup and restore plans in Rust (lib/rust/crates/backup). [BackupPolicy] wraps it.",
    module: "backup",
    file: "packets",
    methods: &[call("::backup::call")],
    ..CLASS
};

pub const SETTINGS_NATIVE: Class = Class {
    name: "SettingsNative",
    doc: "The settings catalogue and values in Rust (lib/rust/crates/settings). [SettingsRules] \
          wraps it.",
    module: "settings",
    file: "packets",
    methods: &[call("::settings::call")],
    ..CLASS
};

pub const EXTENSION_POLICY_NATIVE: Class = Class {
    name: "ExtensionPolicyNative",
    doc: "Extension decisions in Rust (lib/rust/crates/extensions). [ExtensionPolicy] wraps it.",
    module: "extensions",
    file: "packets",
    methods: &[call("::extensions::call")],
    ..CLASS
};
