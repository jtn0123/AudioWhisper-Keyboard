use crate::protocol::*;
use keys::codes::*;
use std::collections::BTreeMap;

#[derive(Debug)]
struct Press {
    blocked: bool,
    generation: i64,
}
/// No Java references or callbacks; the host owns key objects and scheduled wakeups.
#[derive(Debug, Default)]
pub struct Dispatcher {
    pressed: BTreeMap<i32, Press>,
    repeatable: Vec<i32>,
    down: (i32, i64),
    up: (i32, i64),
    generation: i64,
}
impl Dispatcher {
    pub fn call(&mut self, operation: i32, v: &[i64]) -> Result<Vec<i64>, &'static str> {
        let need = match operation {
            INIT => 0,
            DOWN => 3,
            UP | DOWN_UP | CANCEL => 2,
            HOLD_RESULT => 4,
            REPEAT_RESULT => 3,
            FACTS => 3,
            DELAYS => 4,
            ACK_DOWN | ACK_UP => 2,
            _ => return Err("unknown dispatcher operation"),
        };
        if v.len() < need {
            return Err("short dispatcher packet");
        }
        let code = v.first().copied().unwrap_or(0) as i32;
        Ok(match operation {
            INIT => {
                self.repeatable = v.iter().map(|v| *v as i32).collect();
                vec![]
            }
            DOWN => {
                if self.pressed.contains_key(&code) {
                    return Ok(vec![]);
                }
                self.generation = self.generation.wrapping_add(1);
                self.pressed.insert(
                    code,
                    Press {
                        blocked: false,
                        generation: self.generation,
                    },
                );
                // The receiver observes the PREVIOUS last-down, then the host acknowledges delivery.
                vec![
                    i64::from(EMIT_DOWN),
                    i64::from(code),
                    v[1],
                    v[2],
                    self.generation,
                ]
            }
            ACK_DOWN | ACK_UP => {
                self.acknowledge(operation == ACK_DOWN, code, v[1]);
                vec![]
            }
            DELAYS => {
                let repeat = repeat_code(code, v[3] != 0);
                vec![
                    long_delay(code, v[1]),
                    repeat_delay(repeat, v[2]),
                    i64::from(repeat),
                ]
            }
            UP => match self.pressed.remove(&code) {
                Some(p) => vec![
                    i64::from(if p.blocked { EMIT_CANCEL } else { EMIT_UP }),
                    i64::from(code),
                ],
                None => vec![],
            },
            CANCEL => {
                if self.pressed.remove(&code).is_some() {
                    vec![i64::from(EMIT_CANCEL), i64::from(code)]
                } else {
                    vec![]
                }
            }
            DOWN_UP => {
                self.pressed.remove(&code);
                vec![i64::from(EMIT_DOWN_UP), i64::from(code)]
            }
            HOLD_RESULT => {
                let Some(p) = self.pressed.get_mut(&code) else {
                    return Ok(vec![]);
                };
                if p.generation != v[3] {
                    return Ok(vec![]);
                }
                if v[1] != 0 {
                    p.blocked = true;
                    vec![]
                } else if self.repeatable.contains(&code) {
                    let repeat = repeat_code(code, v[2] != 0);
                    vec![
                        i64::from(EMIT_REPEAT),
                        i64::from(repeat),
                        i64::from(matches!(code, DELETE | FORWARD_DELETE)),
                    ]
                } else {
                    vec![]
                }
            }
            REPEAT_RESULT => {
                let accepted = self.pressed.get_mut(&code).is_some_and(|p| {
                    if p.generation != v[2] {
                        return false;
                    }
                    p.blocked |= v[1] != 0;
                    v[1] != 0
                });
                vec![i64::from(accepted)]
            }
            FACTS => vec![
                i64::from(self.pressed.contains_key(&code)),
                i64::from(!self.pressed.is_empty()),
                i64::from(shift::is_consecutive(
                    self.down.0,
                    self.down.1,
                    code,
                    v[1],
                    v[2],
                )),
                i64::from(shift::is_consecutive(
                    self.up.0, self.up.1, code, v[1], v[2],
                )),
                i64::from(self.down.0 == code),
                i64::from(self.repeatable.contains(&code)),
                i64::from(self.repeatable.contains(&self.down.0)),
                self.pressed.get(&code).map_or(0, |p| p.generation),
            ],
            _ => unreachable!(),
        })
    }
    pub fn acknowledge(&mut self, down: bool, code: i32, time: i64) {
        if down {
            self.down = (code, time)
        } else {
            self.up = (code, time)
        }
    }
}
pub fn long_delay(code: i32, delay: i64) -> i64 {
    let factor = match code {
        SPACE | CJK_SPACE | SHIFT => 2.5,
        LANGUAGE_SWITCH => 2.0,
        _ => 1.0,
    };
    (delay as f32 * factor) as i64
}
pub fn repeat_delay(code: i32, delay: i64) -> i64 {
    let factor = if matches!(code, DELETE_WORD | FORWARD_DELETE_WORD | UNDO | REDO) {
        5.0
    } else {
        1.0
    };
    (delay as f32 * factor) as i64
}
pub fn repeat_code(code: i32, words: bool) -> i32 {
    match (code, words) {
        (DELETE, true) => DELETE_WORD,
        (FORWARD_DELETE, true) => FORWARD_DELETE_WORD,
        _ => code,
    }
}
