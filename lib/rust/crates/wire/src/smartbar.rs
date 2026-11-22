//! `SmartbarRules.actions`: what each smartbar action shows, which icon, label and state, and
//! what the arrow toggles, answered by `render::smartbar_actions`. Android supplies only what its
//! key evaluator knows of each action and draws the icon each answer names.

/// The icons an action shows.
pub mod icon {
    crate::declare! {
        /// The icon an action shows; Android draws each from its own vector.
        pub const ICONS: object SmartbarIcon(i32) {
            /// None: the action shows its label.
            NONE = 0,
            /// The key's own icon, from its key data.
            KEY = 1,
            /// The arrow, pointing left.
            ARROW_LEFT = 2,
            /// The arrow, pointing right.
            ARROW_RIGHT = 3,
            /// The extended actions fold away.
            UNFOLD_LESS = 4,
            /// The extended actions unfold.
            UNFOLD_MORE = 5,
            /// Incognito, in the arrow's place.
            INCOGNITO = 6,
            /// Reading aloud stops.
            STOP = 7,
        }
    }
}

/// What an action slot holds.
pub mod kind {
    crate::declare! {
        /// What an action slot holds.
        pub const KINDS: object SmartbarActionKind(i32) {
            /// Nothing: the arrow's slot, or one never placed.
            NONE = 0,
            /// A key.
            KEY = 1,
            /// Text to type.
            TEXT = 2,
        }
    }
}

/// The facts every action reads that the smartbar's config does not carry.
pub mod bar {
    crate::declare! {
        /// What every action reads beyond the smartbar's config, then a `SmartbarAction` row for
        /// each action.
        pub const FIELDS: fields SmartbarBar {
            /// The `IncognitoDisplayMode` ordinal: 0 replaces the arrow, 1 shows behind the keys.
            INCOGNITO_DISPLAY,
            /// A flag: incognito is on.
            INCOGNITO,
            /// A flag: dictation is listening.
            LISTENING,
            /// A flag: the dictation model is warming up.
            WARMING,
            /// The microphone's level, from 0 to 1, as Kotlin spells a `Float`.
            LEVEL,
        }
    }
}

/// What Android knows of one action.
pub mod action {
    crate::declare! {
        /// One action, as Android's key evaluator sees it.
        pub const FIELDS: fields SmartbarAction {
            /// A `SmartbarActionKind`.
            KIND,
            /// Its key code.
            CODE,
            /// A flag: the evaluator enables its key.
            ENABLED,
            /// A flag: its key has an icon.
            KEY_ICON,
            /// The text it types.
            TEXT,
        }
    }
}

/// What the smartbar shows.
pub mod plan {
    crate::declare! {
        /// What the arrow does, then a `SmartbarActionPlan` row for each action.
        pub const FIELDS: fields SmartbarPlan {
            /// A flag: the actions show expanded, so the toggle collapses them.
            EXPANDED,
            /// A flag: the toggle flips the extended actions, else the shared ones.
            TOGGLES_EXTENDED,
        }
    }
}

/// What one action shows.
pub mod shown {
    crate::declare! {
        /// What one action shows.
        pub const FIELDS: fields SmartbarActionPlan {
            /// The key code the renderer reports a tap on.
            CODE,
            /// The `RenderProtocol` `BAR_` flags and the microphone level.
            FLAGS,
            /// A `SmartbarIcon`.
            ICON,
            /// The label, when it shows one of its own.
            LABEL,
            /// A flag: show the key's own label instead.
            KEY_LABEL,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &icon::ICONS,
    &kind::KINDS,
    &bar::FIELDS,
    &action::FIELDS,
    &plan::FIELDS,
    &shown::FIELDS,
];
