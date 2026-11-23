//! `ImeThemeNative`: the keyboard's theme, which Rust chooses from the facts Android reports and
//! the theme extensions Android indexed, answered by the `imetheme` crate.
//!
//! Android reads what Rust asks it to load (an extension's archive, a stylesheet in it) and says
//! whether it parsed; Rust says which theme is then active.

crate::declare! {
    /// The operations `ImeThemeNative.call` takes, each answered with a [`step`] row.
    pub const OPS: enum ImeThemeOp(str) {
        /// An [`update`] row, then `KEYBOARDS` keyboard extension ids, then for each of `THEMES`
        /// theme extensions an [`extension`] row and a [`component`] row for each of its
        /// components.
        UPDATE = "update",
        /// A [`loaded`] row: what came of the load the last step asked for.
        LOADED = "loaded",
    }
}

crate::declare! {
    /// The rules `ImeThemeNative.rules` takes, which need no session.
    pub const RULES: enum ImeThemeRule(str) {
        /// Nothing: the base stylesheet's JSON, the theme shown when no other is.
        BASE = "base",
        /// An [`inline`] row, then a palette: its names, then each name's light colour, then
        /// each name's dark colour. Answered with a [`chip`] row.
        INLINE = "inline",
        /// An [`accent`] row: the accent colour a theme's palette is made from.
        PRIMARY = "primary",
        /// A [`window`] row, then a palette as `INLINE` takes it. Answered with a [`backdrop`]
        /// row.
        WINDOW = "window",
    }
}

/// What a theme update is made from.
pub mod update {
    crate::declare! {
        /// The facts a theme update is chosen from.
        pub const FIELDS: fields ImeThemeUpdate {
            /// A flag: the theme extensions changed since the last update, so drop every loaded
            /// theme first.
            CLEAR,
            /// The `ThemeMode` entry's name.
            MODE,
            /// The day and night themes the settings name: an extension id and a component id.
            DAY_EXTENSION,
            DAY_COMPONENT,
            NIGHT_EXTENSION,
            NIGHT_COMPONENT,
            /// Sunrise, sunset and now, as nanoseconds of the day.
            SUNRISE,
            SUNSET,
            NOW,
            /// A flag: the system is in night mode.
            SYSTEM_NIGHT,
            /// A flag: the theme manager previews a theme by name, `PREVIEW_EXTENSION` and
            /// `PREVIEW_COMPONENT`.
            PREVIEW,
            PREVIEW_EXTENSION,
            PREVIEW_COMPONENT,
            /// A flag: the theme editor previews a theme it holds, which wins over everything.
            PREVIEW_INFO,
            /// How many keyboard extension ids and theme extensions follow.
            KEYBOARDS,
            THEMES,
        }
    }
}

/// A theme extension, as Android indexed it.
pub mod extension {
    crate::declare! {
        /// A theme extension: its id, whether it has an archive to load, and how many
        /// [`super::component`] rows follow.
        pub const FIELDS: fields ImeThemeExtension {
            ID,
            /// A flag: it has a source archive.
            SOURCE,
            COMPONENTS,
        }
    }
}

/// One theme in an extension.
pub mod component {
    crate::declare! {
        /// A theme component: its id and the stylesheet path it names, empty if none.
        pub const FIELDS: fields ImeThemeComponent {
            ID,
            STYLESHEET,
        }
    }
}

/// What Android does next.
pub mod step {
    crate::declare! {
        /// What Android does next: a [`super::kind`] code, then what it needs for it.
        pub const FIELDS: fields ImeThemeStep {
            KIND,
            /// A flag: every loaded theme was dropped, so Android drops what it held for them.
            CLEARED,
            /// The loaded theme to show, for `CACHED` and `ACTIVE`.
            SLOT,
            /// For `LOAD`: the theme extension's and the component's indices in the update, and
            /// the stylesheet's path in the archive.
            EXTENSION,
            COMPONENT,
            PATH,
        }
    }
}

/// What a step asks of Android.
pub mod kind {
    crate::declare! {
        /// A [`super::step`] row's `KIND`.
        pub const CODES: object ImeThemeStepKind(i32) {
            /// Show the theme the editor previews.
            PREVIEW = 0,
            /// Show the loaded theme `SLOT`.
            CACHED = 1,
            /// Change nothing: the theme named is not one that can be loaded.
            KEEP = 2,
            /// Load the stylesheet `PATH` from extension `EXTENSION`'s archive, as component
            /// `COMPONENT`, and say what came of it with `LOADED`.
            LOAD = 3,
            /// The load worked: show it as loaded theme `SLOT`.
            ACTIVE = 4,
            /// The load failed: show the base theme.
            FAILED = 5,
        }
    }
}

/// What came of a load.
pub mod loaded {
    crate::declare! {
        /// What came of the load a `LOAD` step asked for.
        pub const FIELDS: fields ImeThemeLoaded {
            /// A flag: the stylesheet was read and parsed.
            OK,
        }
    }
}

/// What the inline autofill chips are styled from.
pub mod inline {
    crate::declare! {
        /// The active theme and the keyboard's state, which the chips' style is queried in.
        pub const FIELDS: fields ImeThemeInline {
            /// The active stylesheet's JSON.
            STYLESHEET,
            /// The keyboard's mode and shift state, as the theme's attributes name them.
            MODE,
            SHIFT,
        }
    }
}

/// The inline autofill chips' colours.
pub mod chip {
    crate::declare! {
        /// The chips' background and foreground, as ARGB.
        pub const FIELDS: fields ImeThemeChip {
            BACKGROUND,
            FOREGROUND,
        }
    }
}

/// What the keyboard window's background is styled and sized from.
pub mod window {
    crate::declare! {
        /// The active theme, the keyboard's state, and the background image's and surface's
        /// sizes in pixels (0 while unknown).
        pub const FIELDS: fields ImeThemeWindow {
            /// The active stylesheet's JSON.
            STYLESHEET,
            /// The keyboard's mode and shift state, as the theme's attributes name them.
            MODE,
            SHIFT,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            SURFACE_WIDTH,
            SURFACE_HEIGHT,
        }
    }
}

/// The keyboard window's background.
pub mod backdrop {
    crate::declare! {
        /// The background's colour (ARGB), its image's path in the theme's directory (empty if
        /// none), flags for dark navigation bar buttons and for keeping the bar readable over
        /// the image, and where the image is drawn on the surface.
        pub const FIELDS: fields ImeThemeBackdrop {
            BACKGROUND,
            IMAGE,
            LIGHT_BARS,
            CONTRAST,
            LEFT,
            TOP,
            RIGHT,
            BOTTOM,
        }
    }
}

/// The accent a palette is made from.
pub mod accent {
    crate::declare! {
        /// The accent the settings name, if they name one, and the system's.
        pub const FIELDS: fields ImeThemeAccent {
            /// A flag: the settings name an accent, `PREFERRED`.
            SPECIFIED,
            /// ARGB colours.
            PREFERRED,
            SYSTEM,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &RULES,
    &update::FIELDS,
    &extension::FIELDS,
    &component::FIELDS,
    &step::FIELDS,
    &kind::CODES,
    &loaded::FIELDS,
    &inline::FIELDS,
    &chip::FIELDS,
    &window::FIELDS,
    &backdrop::FIELDS,
    &accent::FIELDS,
];
