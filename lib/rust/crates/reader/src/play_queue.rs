//! Playback-frame policy. Managed callbacks are opaque block ids and execute after unlocking.
use std::collections::{BTreeMap, VecDeque};

/// Queue look-ahead and playback-head observation cadence, preserved from the managed reader.
pub const LOOKAHEAD: i64 = 2;
pub const FOLLOW_MS: i64 = 40;
#[derive(Debug)]
struct Block {
    id: i32,
    start: Option<i64>,
    end: Option<i64>,
    pending: bool,
}
#[derive(Debug, Default)]
pub struct PlayQueue {
    blocks: VecDeque<Block>,
    speech: BTreeMap<i32, bool>,
}
impl PlayQueue {
    /// System TTS progress: native parsing ignores late/stopped/duplicate callbacks.
    pub fn system(&mut self, op: i32, id: i32, text: &[u16]) -> Result<Vec<i64>, &'static str> {
        let mut result = vec![-1, 0, self.speech.len() as i64];
        match op {
            0 => {
                self.speech.insert(id, true);
            }
            1 | 2 => {
                let split = text.iter().position(|&c| c == u16::from(b':'));
                let key = chars::text::to_int_or_null(&text[..split.unwrap_or(text.len())]);
                if let Some(key) = key {
                    if op == 1 {
                        if self
                            .speech
                            .get_mut(&key)
                            .is_some_and(|pending| std::mem::replace(pending, false))
                        {
                            result[0] = i64::from(key);
                            result[1] = 1;
                        }
                    } else if split.is_some_and(|i| text[i + 1..] == [108, 97, 115, 116])
                        && self.speech.remove(&key).is_some()
                    {
                        result[0] = i64::from(key);
                        result[1] = 1;
                    }
                }
            }
            3 => {
                result[1] = self.speech.len() as i64;
                self.speech.clear();
            }
            4 => {}
            _ => return Err("Unknown speech operation"),
        }
        result[2] = self.speech.len() as i64;
        Ok(result)
    }
    pub fn call(&mut self, op: i32, id: i32, frame: i64) -> Result<Vec<i64>, &'static str> {
        let mut result = vec![];
        match op {
            0 => self.blocks.push_back(Block {
                id,
                start: None,
                end: None,
                pending: true,
            }),
            1 | 2 => {
                if let Some(b) = self.blocks.iter_mut().find(|b| b.id == id) {
                    b.start.get_or_insert(frame);
                    if op == 2 {
                        b.end = Some(frame);
                    }
                }
            }
            3 => {
                for b in &mut self.blocks {
                    let Some(start) = b.start else {
                        break;
                    };
                    let over = b.end.is_some_and(|end| frame >= end);
                    if frame <= start && !over {
                        break;
                    }
                    if b.pending {
                        result.push(i64::from(b.id));
                        b.pending = false;
                    }
                }
                let mut finished = 0;
                while self
                    .blocks
                    .front()
                    .is_some_and(|b| b.end.is_some_and(|end| frame >= end))
                {
                    self.blocks.pop_front();
                    finished += 1;
                }
                result.insert(0, finished);
            }
            4 => {
                result.push(self.blocks.len() as i64);
                self.blocks.clear();
            }
            5 => result.push(self.blocks.len() as i64),
            _ => return Err("Unknown playback operation"),
        }
        Ok(result)
    }
}
