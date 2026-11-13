//! TextKeyboardSwipe's policy. Outputs describe editor operations; no editor or platform state lives here.
use crate::{
    gesture_config::GestureConfig, keyboard_touch::Pointer, protocol::*, scene::Scene,
    touch::Action,
};
use gestures::swipe::{Direction, Swipe};
use keys::codes::*;
pub(crate) fn handle(
    scene: &Scene,
    p: &mut Pointer,
    s: &Swipe,
    up: bool,
    c: &GestureConfig,
    shift: bool,
    out: &mut Vec<Action>,
) -> bool {
    let Some(initial) = p.initial.and_then(|i| scene.keys.get(i)) else {
        return false;
    };
    let active = p.active.and_then(|i| scene.keys.get(i));
    match initial.code {
        DELETE => delete(p, s, up, c, shift, out),
        SPACE | CJK_SPACE => space(p, s, up, c, out),
        SHIFT if !up && active.is_some_and(|k| k.code == SPACE || k.code == CJK_SPACE) => {
            space(p, s, up, c, out)
        }
        SHIFT if up && active.is_none_or(|k| k.code != SHIFT) => {
            if let Some(key) = p.active {
                out.push(Action(KEY_UP, key as i32, p.id));
            }
            out.push(Action(KEY_CANCEL_CODE, SHIFT, p.id));
            true
        }
        code if code > SPACE && scene.popup.is_none() && !c.glide && !p.moved && up => {
            let pref = match s.direction {
                Direction::Up => PREF_SWIPE_UP,
                Direction::Down => PREF_SWIPE_DOWN,
                Direction::Left => PREF_SWIPE_LEFT,
                Direction::Right => PREF_SWIPE_RIGHT,
                _ => return false,
            };
            execute(c, pref, p.id, out)
        }
        _ => false,
    }
}
fn execute(c: &GestureConfig, pref: i32, id: i32, out: &mut Vec<Action>) -> bool {
    if c.action(pref as usize) == "NO_ACTION" {
        false
    } else {
        out.push(Action(SWIPE_ACTION, pref, id));
        true
    }
}
fn delete(
    p: &Pointer,
    s: &Swipe,
    up: bool,
    c: &GestureConfig,
    shift: bool,
    out: &mut Vec<Action>,
) -> bool {
    let action = c.action(PREF_DELETE_LEFT as usize);
    let chars = ["DELETE_CHARACTERS_PRECISELY", "SELECT_CHARACTERS_PRECISELY"].contains(&action);
    let words = ["DELETE_WORDS_PRECISELY", "SELECT_WORDS_PRECISELY"].contains(&action);
    if c.gif && !up {
        if !chars && !words {
            return false;
        }
        if s.rel_x != 0 {
            out.push(Action(GESTURE_FEEDBACK, DELETE, p.id));
        }
        // GIF search restores its query as the finger moves back. The last field means 'starts'.
        out.push(Action(
            if chars {
                GIF_DELETE_CHARACTERS
            } else {
                GIF_DELETE_WORDS
            },
            s.abs_x,
            i32::from(!p.moved),
        ));
        return true;
    }
    if c.raw {
        return false;
    }
    if !up && (chars || words) {
        if s.rel_x != 0 {
            out.push(Action(GESTURE_FEEDBACK, DELETE, p.id));
        }
        let count = if chars {
            s.abs_x.wrapping_neg().wrapping_sub(1)
        } else {
            (s.abs_x.wrapping_neg() / 2).wrapping_sub(1)
        };
        out.push(Action(
            if chars {
                SELECT_CHARACTERS
            } else {
                SELECT_WORDS
            },
            count,
            i32::from(shift),
        ));
        true
    } else if up && s.direction == Direction::Left && action == "DELETE_WORD" {
        execute(c, PREF_DELETE_LEFT, p.id, out)
    } else {
        false
    }
}
fn space(p: &mut Pointer, s: &Swipe, up: bool, c: &GestureConfig, out: &mut Vec<Action>) -> bool {
    if c.gif && !up {
        return true;
    }
    let pref = match s.direction {
        Direction::Left => PREF_SPACE_LEFT,
        Direction::Right => PREF_SPACE_RIGHT,
        _ => {
            if up && s.abs_y < -6 {
                out.push(Action(SWIPE_ACTION, PREF_SPACE_UP, p.id));
                return true;
            }
            return false;
        }
    };
    let cursor = if pref == PREF_SPACE_LEFT {
        "MOVE_CURSOR_LEFT"
    } else {
        "MOVE_CURSOR_RIGHT"
    };
    if !up {
        if c.action(pref as usize) == cursor {
            let count = s.rel_x.wrapping_abs().wrapping_sub(i32::from(!p.moved));
            if count > 0 {
                out.push(Action(GESTURE_FEEDBACK, SPACE, p.id));
                if !p.mass {
                    p.mass = true;
                    out.push(Action(MASS_BEGIN, 0, p.id));
                }
                out.push(Action(
                    if pref == PREF_SPACE_LEFT {
                        CURSOR_LEFT
                    } else {
                        CURSOR_RIGHT
                    },
                    count,
                    p.id,
                ));
            }
            true
        } else {
            c.action(pref as usize) != "NO_ACTION"
        }
    } else if c.action(pref as usize) != cursor {
        execute(c, pref, p.id, out)
    } else {
        false
    }
}
