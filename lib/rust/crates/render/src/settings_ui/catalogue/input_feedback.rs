//! InputFeedbackScreen (`iconSpaceReserved = false`).
use super::{
    Choices, Cond::*, Row, Screen, Slider, entries, heading, list, single, slider, toggle, unit,
};
const AUDIO: Choices = entries(&[
    (
        "RESPECT_SYSTEM_SETTINGS",
        "enum__input_feedback_activation_mode__audio_respect_system_settings",
        "",
    ),
    (
        "IGNORE_SYSTEM_SETTINGS",
        "enum__input_feedback_activation_mode__audio_ignore_system_settings",
        "",
    ),
]);
const HAPTIC: Choices = entries(&[
    (
        "RESPECT_SYSTEM_SETTINGS",
        "enum__input_feedback_activation_mode__haptic_respect_system_settings",
        "",
    ),
    (
        "IGNORE_SYSTEM_SETTINGS",
        "enum__input_feedback_activation_mode__haptic_ignore_system_settings",
        "",
    ),
]);
const VIBRATION: Choices = Choices {
    entries: &[
        (
            "USE_VIBRATOR_DIRECTLY",
            "enum__haptic_vibration_mode__use_vibrator_directly",
            "enum__haptic_vibration_mode__use_vibrator_directly__description",
        ),
        (
            "USE_HAPTIC_FEEDBACK_INTERFACE",
            "enum__haptic_vibration_mode__use_haptic_feedback_interface",
            "enum__haptic_vibration_mode__use_haptic_feedback_interface__description",
        ),
    ],
    selected_only: true,
    emoji: &[],
};
const VOLUME: Slider = single(unit("unit__percent__symbol"));
const DURATION: Slider = Slider {
    hardware: 1,
    preview: true,
    ..single(unit("unit__milliseconds__symbol"))
};
const STRENGTH: Slider = Slider {
    hardware: 2,
    preview: true,
    ..single(unit("unit__percent__symbol"))
};
const AUDIO_ON: super::Cond = Yes("input_feedback__audio_enabled");
const HAPTIC_ON: super::Cond = Yes("input_feedback__haptic_enabled");
const DIRECT: super::Cond = Is(
    "input_feedback__haptic_vibration_mode",
    "USE_VIBRATOR_DIRECTLY",
);
/// One audio or haptic feature switch: `kind` is `audio` or `haptic`, `feat` the feature.
macro_rules! feat {
    ($kind:literal, $feat:literal, $summary:literal, $on:expr) => {
        toggle(
            concat!("input_feedback__", $kind, "_feat_", $feat),
            concat!("pref__input_feedback__", $kind, "_feat_", $feat, "__label"),
            $summary,
        )
        .on($on)
    };
}
const ROWS: &[Row] = &[
    heading("pref__input_feedback__group_audio__label"),
    list(
        "input_feedback__audio_activation_mode",
        "pref__input_feedback__audio_enabled__label",
        &AUDIO,
    )
    .linked(
        "input_feedback__audio_enabled",
        "pref__input_feedback__audio_enabled__summary_disabled",
    ),
    slider(
        "input_feedback__audio_volume",
        "pref__input_feedback__audio_volume__label",
        &VOLUME,
    )
    .on(AUDIO_ON),
    feat!(
        "audio",
        "key_press",
        "pref__input_feedback__any_feat_key_press__summary",
        AUDIO_ON
    ),
    feat!(
        "audio",
        "key_long_press",
        "pref__input_feedback__any_feat_key_long_press__summary",
        AUDIO_ON
    ),
    feat!(
        "audio",
        "key_repeated_action",
        "pref__input_feedback__any_feat_key_repeated_action__summary",
        AUDIO_ON
    ),
    feat!(
        "audio",
        "gesture_swipe",
        "pref__input_feedback__any_feat_gesture_swipe__summary",
        AUDIO_ON
    ),
    feat!(
        "audio",
        "gesture_moving_swipe",
        "pref__input_feedback__audio_feat_gesture_moving_swipe__label",
        AUDIO_ON
    ),
    heading("pref__input_feedback__group_haptic__label"),
    list(
        "input_feedback__haptic_activation_mode",
        "pref__input_feedback__haptic_enabled__label",
        &HAPTIC,
    )
    .linked(
        "input_feedback__haptic_enabled",
        "pref__input_feedback__haptic_enabled__summary_disabled",
    ),
    list(
        "input_feedback__haptic_vibration_mode",
        "pref__input_feedback__haptic_vibration_mode__label",
        &VIBRATION,
    )
    .on(HAPTIC_ON),
    slider(
        "input_feedback__haptic_vibration_duration",
        "pref__input_feedback__haptic_vibration_duration__label",
        &DURATION,
    )
    .on(All(&[HAPTIC_ON, DIRECT, Vibrator])),
    slider(
        "input_feedback__haptic_vibration_strength",
        "pref__input_feedback__haptic_vibration_strength__label",
        &STRENGTH,
    )
    .on(All(&[HAPTIC_ON, DIRECT, Vibrator, Amplitude])),
    feat!(
        "haptic",
        "key_press",
        "pref__input_feedback__any_feat_key_press__summary",
        HAPTIC_ON
    ),
    feat!(
        "haptic",
        "key_long_press",
        "pref__input_feedback__any_feat_key_long_press__summary",
        HAPTIC_ON
    ),
    feat!(
        "haptic",
        "key_repeated_action",
        "pref__input_feedback__any_feat_key_repeated_action__summary",
        HAPTIC_ON
    ),
    feat!(
        "haptic",
        "gesture_swipe",
        "pref__input_feedback__any_feat_gesture_swipe__summary",
        HAPTIC_ON
    ),
    feat!(
        "haptic",
        "gesture_moving_swipe",
        "pref__input_feedback__audio_feat_gesture_moving_swipe__label",
        HAPTIC_ON
    ),
];
pub(super) const SCREEN: Screen = Screen {
    route: "settings/keyboard/input-feedback",
    title: "settings__input_feedback__title",
    icon_space: false,
    preview: true,
    rows: ROWS,
};
