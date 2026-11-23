//! `ImeWindowRules.layout`: where the plain-View root puts the keyboard, its one-handed panels,
//! the inline autofill chips and the background in the input method window, answered by
//! `render::ime_root`. Android reports what it measured and what the settings say; the answer is
//! in pixels, placed as Compose's root placed them.

/// What the root is laid out from.
pub mod facts {
    crate::declare! {
        /// The window's measurements and the settings the layout reads, then the Smartbar's
        /// `smartbar::Config` floats (12, 14 or 17), whose density and row height Rust fills in.
        pub const FIELDS: fields ImeWindowFacts {
            /// The display's density, and the system's, which Compose's pixel and dp sizes were
            /// converted by.
            DENSITY,
            SYSTEM_DENSITY,
            /// The input view's width, and its height's measure spec: a mode (0 unspecified,
            /// 1 at most, 2 exactly) and a size.
            WIDTH,
            HEIGHT_MODE,
            HEIGHT,
            /// Flags: the window is full screen, and its extract view is shown.
            FULLSCREEN,
            EXTRACT_SHOWN,
            /// The safe drawing insets: system bars, cutouts and the IME, each side's largest.
            SAFE_LEFT,
            SAFE_TOP,
            SAFE_RIGHT,
            SAFE_BOTTOM,
            /// A flag: the screen is in landscape.
            LANDSCAPE,
            /// The bottom offset settings, in dp.
            OFFSET_PORTRAIT,
            OFFSET_LANDSCAPE,
            /// One-handed mode: a flag, a flag for the end side, and the scale factor in percent.
            ONE_HANDED,
            ONE_HANDED_END,
            ONE_HANDED_SCALE,
            /// The keyboard height settings, in percent.
            HEIGHT_PORTRAIT,
            HEIGHT_LANDSCAPE,
            /// The display's size, the navigation bar's bottom inset and the status bar's top.
            DISPLAY_WIDTH,
            DISPLAY_HEIGHT,
            NAVIGATION_BAR,
            STATUS_BAR,
            /// The input view's height resources: its smallest and largest fractions and its
            /// base height in pixels.
            MIN_FRACTION,
            MAX_FRACTION,
            BASE_HEIGHT,
            /// The active keyboard's mode (`KeyboardMode.value`) and rows, and the rows of the
            /// last characters keyboard.
            MODE,
            ROWS,
            CHARACTER_ROWS,
            /// Which panel is up: flags for the actions editor and overflow, the UI mode
            /// (`ImeUiMode.value`), and a flag for GIF results showing.
            EDITOR,
            OVERFLOW,
            UI_MODE,
            GIFS,
            /// A flag: a GIF search is being typed.
            SEARCH,
        }
    }
}

/// Where everything goes.
pub mod plan {
    crate::declare! {
        /// Where everything goes, in whole pixels, then the Smartbar's config as the keyboard
        /// draws it. Panel widths of 0 mean no panel; an inline width of 0 means no chips.
        pub const FIELDS: fields ImeWindowPlan {
            /// The input view's height, and the window box's top and height in it.
            VIEW_HEIGHT,
            BOX_TOP,
            BOX_HEIGHT,
            /// The one-handed panel at the start: left, top and size, in the box.
            START_LEFT,
            START_TOP,
            START_WIDTH,
            START_HEIGHT,
            /// The keyboard's box, in the window box.
            KEYBOARD_LEFT,
            KEYBOARD_TOP,
            KEYBOARD_WIDTH,
            KEYBOARD_HEIGHT,
            /// The one-handed panel at the end, in the box.
            END_LEFT,
            END_TOP,
            END_WIDTH,
            END_HEIGHT,
            /// The keyboard surface's top (negative above the box) and height, in the keyboard's
            /// box.
            SURFACE_TOP,
            SURFACE_HEIGHT,
            /// The inline autofill chips' row, in the keyboard's box.
            INLINE_LEFT,
            INLINE_TOP,
            INLINE_WIDTH,
            INLINE_HEIGHT,
            /// The panel showing: 0 none, 1 emoji, 2 actions overflow, 3 actions editor,
            /// 4 clipboard, 5 GIFs.
            PANEL,
            /// A row's and the Smartbar's height in whole pixels, as `FlorisImeSizing.Static`
            /// keeps them.
            ROW_PIXELS,
            SMARTBAR_PIXELS,
            /// A row's height, the keyboard's and the strip's, as the renderer is given them.
            ROW_HEIGHT,
            KEYBOARD_PX,
            STRIP_PX,
            /// The Smartbar's row height in pixels, which its config carries for the actions Rust
            /// plans from the settings' Smartbar rather than a search's.
            SMARTBAR_ROW,
            /// A flag: the extract view sits right above the keyboard, so the keyboard's surface
            /// rises above the window while a finger is down or a popup is open. What it draws
            /// above the keys, a popup or the press bubble, then shows over the extract view.
            RISE,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[&facts::FIELDS, &plan::FIELDS];
