use crate::protocol::*;
use keys::{codes::*, models};

/// The Android host supplies facts once per key. Plans contain no UI objects or editor text.
/// Fields: phase, code, type, register, shift-held, any-held, consecutive-down,
/// uninterrupted, capitalization-behavior, utility-action, GIF route (-1 inactive), alphabetic, forced incognito, one-handed enabled, smartbar enabled.
pub fn route(v: &[i64]) -> Result<Vec<i64>, &'static str> {
    if v.len() != 15 {
        return Err("wrong routing field count");
    }
    let phase = v[0] as i32;
    let code = v[1] as i32;
    let kind = v[2];
    let raw = v[3] as u64;
    let held = v[4] != 0;
    let mut out = Vec::new();
    let mut add = |action: i32, arg: i64| out.extend([i64::from(action), arg, 0]);
    let arrow = (MOVE_END_OF_LINE..=ARROW_LEFT).contains(&code);
    let gif = v[10];
    if phase == PHASE_REPEAT {
        add(FEEDBACK_REPEAT, i64::from(code));
        if arrow && (gif < 0 || gif == 1) {
            add(ARROW, i64::from(code));
            return Ok(out);
        }
    }
    if phase == PHASE_DOWN || phase == PHASE_CANCEL {
        if gif >= 0 && gif != 1 {
            return Ok(out);
        }
        if arrow {
            add(
                if phase == PHASE_DOWN {
                    MASS_BEGIN
                } else {
                    MASS_END
                },
                0,
            );
        } else if code == SHIFT {
            let next = shifted(
                raw,
                if phase == PHASE_CANCEL {
                    shift::Event::ShiftCancel
                } else {
                    shift::Event::ShiftDown
                },
                v,
            );
            add(STATE, (2i64 << 32) | i64::from(next));
        } else if code == VOICE_INPUT {
            if phase == PHASE_DOWN {
                add(STOP_READING, 0);
                add(MIC_DOWN, 0);
            } else {
                add(MIC_UP, 0);
            }
        } else if code == READ_ALOUD {
            if phase == PHASE_DOWN {
                add(STOP_DICTATION, 0);
                add(READ_DOWN, 0);
            } else {
                add(READ_CANCEL, 0);
            }
        }
        return Ok(out);
    }
    if phase != PHASE_UP && phase != PHASE_REPEAT {
        return Err("unknown routing phase");
    }
    if code != DELETE {
        add(CLEAR_UNDO, 0);
    }
    if code != DELETE && code != VOICE_INPUT {
        add(FORGET_DICTATION, 0);
    }
    if gif >= 0 {
        if gif != 1 {
            if gif == 0 {
                match code {
                    DELETE => add(GIF_DELETE_KEY, 0),
                    DELETE_WORD => add(GIF_DELETE_WORD_KEY, 0),
                    SPACE | CJK_SPACE => add(GIF_SPACE, 0),
                    ENTER => {
                        add(GIF_SUBMIT, 0);
                        add(STATE, (3i64 << 32) | 1);
                    }
                    _ => {
                        add(GIF_TYPE, 0);
                        let next = shifted(raw, shift::Event::KeyTyped, v);
                        add(STATE, (2i64 << 32) | i64::from(next));
                    }
                }
            }
            return Ok(out);
        }
        if matches!(code, IME_UI_MODE_MEDIA | IME_UI_MODE_CLIPBOARD) {
            add(GIF_STOP, 0);
        }
    }
    macro_rules! set {
        ($field:expr,$value:expr) => {{
            add(STATE, (($field as i64) << 32) | i64::from($value));
        }};
    }
    match code {
        ARROW_DOWN | ARROW_LEFT | ARROW_RIGHT | ARROW_UP | MOVE_START_OF_PAGE
        | MOVE_END_OF_PAGE | MOVE_START_OF_LINE | MOVE_END_OF_LINE => {
            add(MASS_END, 0);
            add(ARROW, i64::from(code));
        }
        CAPS_LOCK => set!(2, 3),
        SHIFT => {
            let old = models::read(raw, 2).unwrap_or(0);
            if old != 3 && v[5] == 0 && v[7] == 0 {
                set!(2, 0);
            }
        }
        CHAR_WIDTH_SWITCHER => {
            let next = 1 - models::read(raw, 14).unwrap_or(0);
            set!(14, next);
        }
        CHAR_WIDTH_FULL => set!(14, 0),
        CHAR_WIDTH_HALF => set!(14, 1),
        CLIPBOARD_CUT => add(CUT, 0),
        CLIPBOARD_COPY => add(COPY, 0),
        CLIPBOARD_PASTE => add(PASTE, 0),
        CLIPBOARD_SELECT => add(SELECT, 0),
        CLIPBOARD_SELECT_ALL => add(SELECT_ALL, 0),
        CLIPBOARD_CLEAR_HISTORY => add(CLEAR_HISTORY, 0),
        CLIPBOARD_CLEAR_FULL_HISTORY => add(CLEAR_FULL_HISTORY, 0),
        CLIPBOARD_CLEAR_PRIMARY_CLIP => add(CLEAR_CLIP, 0),
        TOGGLE_COMPACT_LAYOUT => add(ONE_HANDED, ((1 - v[13]) << 32) | i64::from(u32::MAX)),
        COMPACT_LAYOUT_TO_LEFT => add(ONE_HANDED, (1 - v[13]) << 32),
        COMPACT_LAYOUT_TO_RIGHT => add(ONE_HANDED, ((1 - v[13]) << 32) | 1),
        DELETE | DELETE_WORD => add(
            if held { DELETE_FORWARD } else { DELETE_BACK },
            i64::from(code == DELETE_WORD),
        ),
        FORWARD_DELETE | FORWARD_DELETE_WORD => {
            add(DELETE_FORWARD, i64::from(code == FORWARD_DELETE_WORD))
        }
        ENTER => add(ENTER_KEY, 0),
        IME_SHOW_UI => add(SHOW_UI, 0),
        IME_HIDE_UI => add(HIDE_UI, 0),
        IME_PREV_SUBTYPE => add(SUBTYPE, -1),
        IME_NEXT_SUBTYPE => add(SUBTYPE, 1),
        IME_UI_MODE_TEXT => set!(3, 0),
        IME_UI_MODE_MEDIA => set!(3, 1),
        IME_UI_MODE_CLIPBOARD => set!(3, 2),
        VOICE_INPUT => add(MIC_UP, 0),
        READ_ALOUD => add(READ_UP, 0),
        LANGUAGE_SWITCH => {
            if matches!(v[9], 1 | 3) {
                add(SUBTYPE, 1);
            } else {
                add(NEXT_IME, 0);
            }
        }
        UNDO => add(UNDO_KEY, 0),
        REDO => add(REDO_KEY, 0),
        SETTINGS => add(SETTINGS_UI, 0),
        SPACE => add(SPACE_KEY, 0),
        SYSTEM_INPUT_METHOD_PICKER => add(IME_PICKER, 0),
        SHOW_SUBTYPE_PICKER => set!(12, 1),
        SYSTEM_PREV_INPUT_METHOD => add(PREV_IME, 0),
        SYSTEM_NEXT_INPUT_METHOD => add(NEXT_IME, 0),
        TOGGLE_SMARTBAR_VISIBILITY => add(SMARTBAR, 1 - v[14]),
        TOGGLE_ACTIONS_OVERFLOW => {
            let n = 1 - models::read(raw, 10).unwrap_or(0);
            set!(10, n);
        }
        TOGGLE_ACTIONS_EDITOR => {
            let n = 1 - models::read(raw, 11).unwrap_or(0);
            set!(11, n);
        }
        TOGGLE_INCOGNITO_MODE => {
            let next = 1 - models::read(raw, 9).unwrap_or(0);
            add(INCOGNITO, (1 - v[12]) | (i64::from(next) << 1));
        }
        TOGGLE_AUTOCORRECT => add(AUTOCORRECT_TOGGLE, 0),
        VIEW_CHARACTERS => set!(0, 0),
        VIEW_SYMBOLS => set!(0, 2),
        VIEW_SYMBOLS2 => set!(0, 3),
        VIEW_NUMERIC => set!(0, 4),
        VIEW_NUMERIC_ADVANCED => set!(0, 5),
        VIEW_PHONE => set!(0, 6),
        VIEW_PHONE2 => set!(0, 7),
        _ => {
            if models::read(raw, 3) == Some(1) {
                add(AUTO_CORRECT, -1);
                add(COMMIT_TEXT, 0);
                return Ok(out);
            }
            let numeric = matches!(models::read(raw, 0), Some(4..=7));
            if numeric {
                if matches!(kind, 0 | 7) || matches!(code, PHONE_PAUSE | PHONE_WAIT) {
                    add(COMMIT_TEXT, 0);
                }
            } else if matches!(kind, 0 | 7) {
                if v[11] == 0 {
                    add(AUTO_CORRECT, 1);
                }
                add(TAP_TYPED, 0);
                add(COMMIT_CHAR, 0);
            } else {
                add(UNKNOWN_KEY, 0);
            }
            let next = shifted(raw, shift::Event::KeyTyped, v);
            add(STATE, (2i64 << 32) | i64::from(next));
        }
    }
    Ok(out)
}

fn shifted(raw: u64, event: shift::Event, v: &[i64]) -> i32 {
    use shift::{Behavior, CapsMode, Facts, State};
    let state = match models::read(raw, 2).unwrap_or(0) {
        0 => State::Unshifted,
        1 => State::ShiftedManual,
        2 => State::ShiftedAutomatic,
        _ => State::CapsLock,
    };
    shift::next(
        state,
        event,
        &Facts {
            behavior: if v[8] == 0 {
                Behavior::DoubleTap
            } else {
                Behavior::Cycle
            },
            caps_mode: CapsMode::None,
            shift_pressed: v[4] != 0,
            any_pressed: v[5] != 0,
            consecutive_down: v[6] != 0,
            uninterrupted: v[7] != 0,
            auto_capitalization: false,
            locale_capitalizes: false,
        },
    ) as i32
}
