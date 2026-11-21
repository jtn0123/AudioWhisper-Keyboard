//! Playback-frame policy. Managed callbacks are opaque block ids and execute after unlocking.
use std::collections::{BTreeMap, VecDeque};
use wire::playback::{self as ops, LAST, SEPARATOR, heard, speech};

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
    /// System TTS progress, a `PlaybackSpeechOp`: native parsing ignores late/stopped/duplicate
    /// callbacks. Answers with a `speech::FIELDS` row.
    pub fn system(&mut self, op: i32, id: i32, text: &[u16]) -> Result<Vec<i64>, &'static str> {
        let mut result = vec![0; speech::SIZE];
        result[speech::BLOCK] = -1;
        match op {
            speech::QUEUE => {
                self.speech.insert(id, true);
            }
            speech::START | speech::END => {
                let separator = SEPARATOR.encode_utf16().next();
                let split = text.iter().position(|&c| Some(c) == separator);
                let key = chars::text::to_int_or_null(&text[..split.unwrap_or(text.len())]);
                if let Some(key) = key {
                    let last = || {
                        split.is_some_and(|i| text[i + 1..].iter().copied().eq(LAST.encode_utf16()))
                    };
                    let matched = if op == speech::START {
                        self.speech
                            .get_mut(&key)
                            .is_some_and(|pending| std::mem::replace(pending, false))
                    } else {
                        last() && self.speech.remove(&key).is_some()
                    };
                    if matched {
                        result[speech::BLOCK] = i64::from(key);
                        result[speech::MATCHED] = 1;
                    }
                }
            }
            speech::STOP => {
                result[speech::MATCHED] = self.speech.len() as i64;
                self.speech.clear();
            }
            speech::STATE => {}
            _ => return Err("Unknown speech operation"),
        }
        result[speech::PENDING] = self.speech.len() as i64;
        Ok(result)
    }
    /// A `PlaybackOp`.
    pub fn call(&mut self, op: i32, id: i32, frame: i64) -> Result<Vec<i64>, &'static str> {
        let mut result = vec![];
        match op {
            ops::ADD => self.blocks.push_back(Block {
                id,
                start: None,
                end: None,
                pending: true,
            }),
            ops::BEGINS | ops::ENDS => {
                if let Some(b) = self.blocks.iter_mut().find(|b| b.id == id) {
                    b.start.get_or_insert(frame);
                    if op == ops::ENDS {
                        b.end = Some(frame);
                    }
                }
            }
            ops::HEARD => {
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
                result.insert(heard::FINISHED, finished);
            }
            ops::CLEAR => {
                result.push(self.blocks.len() as i64);
                self.blocks.clear();
            }
            ops::COUNT => result.push(self.blocks.len() as i64),
            _ => return Err("Unknown playback operation"),
        }
        Ok(result)
    }
}
