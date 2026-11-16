package com.jtn0123.audiowhisper.suggest

/**
 * How readily typing is auto-corrected: the setting `correction__auto_correct`. Each strength's
 * threshold is Rust's (`suggest::auto_correction::Strength`), which a strength reaches as its
 * ordinal, so this order is Rust's (`SuggestRules.strengthNames`, held to it by a test).
 */
enum class AutoCorrectStrength {
    OFF,
    MODEST,
    NORMAL,
    AGGRESSIVE;
}
