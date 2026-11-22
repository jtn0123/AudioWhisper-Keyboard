//! The declared classes, one file each, named after the awnative module that answers them, and
//! the Gradle module each of crates/wire's groups is generated in.

mod clipboard;
mod composing;
mod dictation;
mod dictation_session;
mod editor;
mod emoji;
mod gestures;
mod gif;
mod giphy;
mod glide;
mod input;
mod key_graph;
mod keyboard_renderer;
mod keys;
mod latin;
mod layout;
mod learner;
mod packets;
mod popup;
mod reader;
mod reader_runtime;
mod render;
mod rules;
mod screen;
mod sherpa;
mod shift;
mod smartbar;
mod spacing;
mod suggest;
mod swipe;
mod text;
mod tts;
mod word_stores;

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
    gif::GIF_NATIVE,
    composing::COMPOSING_NATIVE,
    giphy::GIPHY_NATIVE,
    emoji::EMOJI_NATIVE,
    spacing::SPACING_NATIVE,
    word_stores::WORD_STORES_NATIVE,
    latin::LATIN_SESSION_NATIVE,
    latin::NLP_NATIVE,
    learner::SUGGESTER_NATIVE,
    learner::NATIVE_LEARNER_NATIVE,
    learner::NATIVE_GLIDE_NATIVE,
    swipe::SWIPE_NATIVE,
    keys::KEYS_NATIVE,
    text::JAVA_TEXT_NATIVE,
    reader::READER_NATIVE,
    reader_runtime::READER_WALK,
    reader_runtime::PLAYBACK_NATIVE,
    input::INPUT_RULES,
    input::INPUT_NATIVE,
    glide::GLIDE_NATIVE,
    gestures::GESTURES_NATIVE,
    popup::POPUP_NATIVE,
    suggest::SUGGEST_NATIVE,
    smartbar::SMARTBAR_NATIVE,
    key_graph::KEY_GRAPH_NATIVE,
    render::RENDER_NATIVE,
    dictation_session::DICTATION_SESSION_NATIVE,
    editor::EDITOR_NATIVE,
    sherpa::SHERPA_NATIVE,
    tts::TTS_NATIVE,
];

/// Every generated api over those classes.
pub const APIS: &[Api] = &[
    shift::SHIFT_RULES,
    screen::SCREEN_WALK_RULES,
    dictation::DICTATION_RULES,
    dictation::MIC_KEY_STATE,
    gif::GIF_RULES,
    gif::GIF_RESULT_KEYS,
    composing::COMPOSING_RULES,
    giphy::GIPHY_RULES,
    emoji::EMOJI_TABLE,
    emoji::EMOJI_RULES,
    spacing::SPACING_RULES,
    word_stores::WORD_STORE_RULES,
    word_stores::LEARNED_WORD_STORE,
    word_stores::KEPT_WORD_STORE,
    word_stores::PERSONAL_WORD_STORE,
    latin::LATIN_SESSION,
    latin::NLP_SESSION,
    latin::NLP_NAMES,
    learner::LEARNER_API,
    learner::NATIVE_GLIDE,
    swipe::SWIPE_RULES,
    keys::KEY_GEOMETRY,
    text::JAVA_TEXT,
    reader::READER_RULES,
    reader::SPEAKER_KEY_STATE,
    reader_runtime::READER_TIMING,
    reader_runtime::PLAYBACK_QUEUE,
    input::INPUT_DISPATCH,
    glide::GLIDE_WORDS,
    glide::GLIDE_INDEX,
    glide::GLIDE_QUERY,
    gestures::SWIPE_DETECTOR,
    gestures::GLIDE_DETECTOR,
    popup::POPUP_GRID,
    popup::POPUP_ORDER,
    suggest::SUGGEST_RULES,
    suggest::QWERTY_LAYOUT,
    suggest::TAP_POSITIONS,
    smartbar::SMARTBAR_RULES,
    key_graph::KEY_GRAPH,
    keyboard_renderer::KEYBOARD_RENDERER,
    dictation_session::DICTATION_SESSION_PLAN,
    sherpa::DICTATION_AUDIO,
    tts::SHERPA_TTS,
];

/// Every generated top-level record, enum and interface, with the declarations file and the
/// module it is in.
pub const TYPES: &[(&str, Module, Type)] = &[
    emoji::EMOJI_ENTRY,
    emoji::EMOJI_ENTRY_SET,
    emoji::EMOJI_TONES,
    reader_runtime::READER_TREE,
    glide::GLIDE_CHAR,
    glide::GLIDE_KEYS,
    glide::GLIDE_SCORES,
    gestures::SWIPE_DIRECTION,
    gestures::SWIPE_RECORD,
    gestures::GLIDE_OUTCOME,
    popup::POPUP_RECT,
    smartbar::TEXT_FIT,
    tts::TTS_AUDIO,
];

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
        groups: wire::typing::GROUPS,
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
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::smartbar::GROUPS,
    },
    Wire {
        module: Module::Rust,
        internal: false,
        groups: wire::key_graph::GROUPS,
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
