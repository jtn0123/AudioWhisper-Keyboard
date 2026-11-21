//! Session decisions. Android executes the ordered effects after the native lock is released.
use crate::mic_key::{Action, MicKey, Status};
use wire::dictation::{
    self as events, effect, model, model_state, phase, plan, problem, ready, status,
};
use wire::flag;

pub const KEEP_LOADED_MS: i64 = 300_000;
pub const KEEP_HIDDEN_MS: i64 = 60_000;

#[derive(Debug, Default)]
pub struct Session {
    pub generation: i32,
    /// A `DictationStatus`.
    status: i32,
    /// A `DictationPhase`.
    pub phase: &'static str,
    warming: bool,
    level: String,
    preview: Vec<u16>,
    released: bool,
    previewing: bool,
    /// A `DictationModelState`.
    model: i32,
    shown: bool,
    idle_since: i64,
    typed: bool,
    failures: i32,
    explained: bool,
    mic: MicKey,
    composition: crate::compose::Composition,
}

pub fn release_at(idle: i64, now: i64, shown: bool) -> i64 {
    let kept = idle.wrapping_add(KEEP_LOADED_MS);
    if shown {
        kept
    } else {
        kept.min(now.wrapping_add(KEEP_HIDDEN_MS))
    }
}
pub fn ending(typed: bool, failures: i32) -> &'static str {
    if typed {
        ""
    } else if failures > 0 {
        problem::FAILED
    } else {
        problem::NOT_CAUGHT
    }
}
fn text(s: impl ToString) -> Vec<u16> {
    s.to_string().encode_utf16().collect()
}

impl Session {
    pub fn speech(&mut self, id: i32, detected: bool) {
        if self.listening(id) {
            self.phase = if detected {
                phase::SPEECH
            } else {
                phase::TRAILING_SILENCE
            };
        }
    }
    /// Answers a `DictationModelOp`.
    pub fn model(&mut self, operation: &str) -> Result<i32, String> {
        match operation {
            model::GET => Ok(self.model),
            model::LOADED => {
                self.model = model_state::LOADED;
                Ok(0)
            }
            model::FAILED => {
                self.model = model_state::FAILED;
                Ok(0)
            }
            model::RETRY => {
                if self.model == model_state::FAILED {
                    self.model = model_state::NONE;
                }
                Ok(0)
            }
            model::IS_LOADED => Ok(i32::from(self.model == model_state::LOADED)),
            model::CLOSE => {
                let loaded = self.model == model_state::LOADED;
                if loaded {
                    self.model = model_state::NONE;
                }
                Ok(i32::from(loaded))
            }
            _ => Err("unknown model operation".into()),
        }
    }
    pub fn claim_preview(&mut self) -> bool {
        if self.previewing || self.released {
            false
        } else {
            self.previewing = true;
            true
        }
    }
    pub fn complete_preview(&mut self) {
        self.previewing = false;
    }
    pub fn new() -> Self {
        Self {
            shown: true,
            phase: phase::IDLE,
            level: "0.0".into(),
            ..Self::default()
        }
    }
    fn current(&self, id: i32) -> bool {
        id == self.generation && self.status != status::IDLE
    }
    fn listening(&self, id: i32) -> bool {
        id == self.generation && self.status == status::LISTENING
    }
    fn schedule(&self, now: i64, effects: &mut Vec<Vec<u16>>) {
        effects.extend([
            text(effect::SCHEDULE),
            text(release_at(self.idle_since, now, self.shown)),
        ]);
    }
    fn idle(&mut self, now: i64, effects: &mut Vec<Vec<u16>>) {
        self.status = status::IDLE;
        self.level = "0.0".into();
        self.warming = false;
        self.preview.clear();
        self.idle_since = now;
        self.schedule(now, effects);
    }
    fn abandon(&mut self, now: i64, effects: &mut Vec<Vec<u16>>) {
        if self.status == status::IDLE {
            return;
        }
        self.generation = self.generation.wrapping_add(1);
        effects.push(text(effect::DISCARD));
        self.phase = phase::CANCELLED;
        self.idle(now, effects);
    }
    fn finish(&mut self, effects: &mut Vec<Vec<u16>>) {
        effects.push(text(effect::FLUSH));
        self.status = status::FINISHING;
        self.level = "0.0".into();
        self.phase = phase::FLUSHING;
        effects.extend([text(effect::BARRIER), text(self.generation)]);
    }
    fn start(effects: &mut Vec<Vec<u16>>) {
        effects.push(text(effect::CHECK));
    }
    fn toggle(&mut self, effects: &mut Vec<Vec<u16>>) {
        match self.status {
            status::IDLE => Self::start(effects),
            status::LISTENING => self.finish(effects),
            _ => {}
        }
    }
    /// `now` and `id` are supplied by executors; no wall-clock or Android dependency.
    pub fn call(
        &mut self,
        op: &str,
        now: i64,
        id: i32,
        fields: &[Vec<u16>],
    ) -> Result<Vec<Vec<u16>>, String> {
        let s = |i: usize| {
            fields
                .get(i)
                .map(|v| String::from_utf16_lossy(v))
                .ok_or("missing dictation field".to_owned())
        };
        if self.released {
            return Ok(Vec::new());
        }
        let mut effects = Vec::new();
        if !self.released {
            match op {
                events::PRESS => {
                    let status = match self.status {
                        status::IDLE => Status::Idle,
                        status::LISTENING => Status::Listening,
                        _ => Status::Finishing,
                    };
                    if self.mic.press(status, now) == Action::Start {
                        Self::start(&mut effects);
                    }
                }
                events::RELEASE_MIC => match self.mic.release(now) {
                    Action::Finish if self.status == status::LISTENING => self.finish(&mut effects),
                    Action::EndHold if self.status == status::LISTENING => {
                        effects.push(text(effect::TICK));
                        self.finish(&mut effects);
                    }
                    Action::Toggle => self.toggle(&mut effects),
                    _ => {}
                },
                events::READY => self.ready(&mut effects, fields)?,
                events::OPENED => {
                    self.typed = false;
                    self.failures = 0;
                    self.composition.reset();
                    self.explained = false;
                    self.status = status::LISTENING;
                    self.phase = phase::LISTENING;
                    effects.extend([text(effect::PRELOAD), text(self.generation)]);
                }
                events::OPEN_ERROR => {
                    self.phase = phase::ERROR;
                    effects.extend([text(effect::PROBLEM), text(problem::MIC_BUSY)]);
                }
                events::STOP if self.status == status::LISTENING => {
                    self.explained = true;
                    self.finish(&mut effects);
                }
                events::CANCEL => self.abandon(now, &mut effects),
                events::VISIBLE => {
                    self.shown = flag::decode(fields.first().ok_or("missing visibility")?);
                    if self.status == status::IDLE && self.idle_since != 0 {
                        self.schedule(now, &mut effects);
                    }
                }
                events::RELEASE => {
                    self.abandon(now, &mut effects);
                    self.released = true;
                    effects.extend([text(effect::REMOVE_UNLOAD), text(effect::CLOSE_WORKER)]);
                }
                events::FREE if self.status == status::IDLE => {
                    effects.extend([text(effect::REMOVE_UNLOAD), text(effect::UNLOAD)])
                }
                events::UNLOAD => effects.push(text(effect::UNLOAD)),
                events::WARMING if self.current(id) => {
                    self.warming = true;
                    self.phase = phase::WARMING;
                }
                events::WARMED => {
                    self.warming = false;
                    if self.phase == phase::WARMING {
                        self.phase = phase::LISTENING;
                    }
                }
                events::MODEL_ERROR if self.current(id) => {
                    effects.extend([text(effect::PROBLEM), text(problem::MODEL_FAILED)]);
                    self.abandon(now, &mut effects);
                    self.phase = phase::ERROR;
                }
                events::MIC_ERROR if self.listening(id) => {
                    effects.extend([text(effect::PROBLEM), text(problem::MIC_STOPPED)]);
                    self.explained = true;
                    self.finish(&mut effects);
                }
                events::LEVEL if self.listening(id) => self.level = s(0)?,
                events::PREVIEW if self.listening(id) => {
                    self.preview = chars::trim(fields.first().ok_or("missing preview")?).to_vec()
                }
                events::VAD_ON if self.listening(id) => self.phase = phase::SPEECH,
                events::VAD_OFF if self.listening(id) => self.phase = phase::TRAILING_SILENCE,
                events::FINALIZING if self.current(id) => self.phase = phase::FINALIZING,
                events::FINAL if id == self.generation => {
                    self.preview.clear();
                    if flag::decode(fields.first().ok_or("missing dictation field")?) {
                        let raw = fields.get(1).ok_or("missing result")?;
                        if !chars::trim(raw).is_empty() {
                            self.typed = true;
                            effects.extend([text(effect::TIDY), raw.clone()]);
                        }
                    } else {
                        self.failures += 1;
                    }
                }
                events::FINISHED if id == self.generation => {
                    if !self.explained {
                        let msg = ending(self.typed, self.failures);
                        if !msg.is_empty() {
                            effects.extend([text(effect::PROBLEM), text(msg)]);
                        }
                    }
                    self.phase = phase::COMMITTED;
                    self.idle(now, &mut effects);
                }
                events::TIDIED => effects.extend(
                    self.composition
                        .prepare(fields.first().ok_or("missing tidied phrase")?),
                ),
                events::COMPOSE => {
                    let before = fields.first().ok_or("missing editor context")?.clone();
                    if fields.len() > 1 {
                        self.composition.set_names(&fields[1..]);
                    }
                    effects.extend(self.composition.plan(before));
                }
                events::SNAPSHOT
                | events::STOP
                | events::FREE
                | events::WARMING
                | events::MODEL_ERROR
                | events::MIC_ERROR
                | events::LEVEL
                | events::PREVIEW
                | events::VAD_ON
                | events::VAD_OFF
                | events::FINALIZING
                | events::FINAL
                | events::FINISHED => {}
                _ => return Err("unknown dictation event".into()),
            }
        }
        let mut row = vec![vec![]; plan::SIZE];
        row[plan::STATUS] = text(self.status);
        row[plan::WARMING] = flag::utf16(self.warming);
        row[plan::LEVEL] = text(&self.level);
        row[plan::PREVIEW] = self.preview.clone();
        row[plan::GENERATION] = text(self.generation);
        row[plan::PHASE] = text(self.phase);
        row.extend(effects);
        Ok(row)
    }

    /// A `READY` row: listen, or explain why not.
    fn ready(&mut self, effects: &mut Vec<Vec<u16>>, fields: &[Vec<u16>]) -> Result<(), String> {
        if fields.len() < ready::SIZE {
            return Err("missing dictation field".to_owned());
        }
        let problem = if !flag::decode(&fields[ready::ALLOWED]) {
            problem::FIELD_OFF
        } else if !flag::decode(&fields[ready::PERMISSION]) {
            problem::ALLOW_MIC
        } else {
            match String::from_utf16_lossy(&fields[ready::MODEL]).as_str() {
                ready::READY => "",
                ready::MISSING => problem::NOT_INSTALLED,
                _ => problem::DAMAGED,
            }
        };
        if problem.is_empty() {
            self.generation = self.generation.wrapping_add(1);
            effects.extend([text(effect::LISTEN), text(self.generation)]);
        } else {
            self.phase = phase::ERROR;
            effects.extend([text(effect::PROBLEM), text(problem)]);
            if problem == problem::ALLOW_MIC {
                effects.push(text(effect::APP_SETTINGS));
            }
            if problem == problem::NOT_INSTALLED || problem == problem::DAMAGED {
                effects.push(text(effect::MODEL_SETTINGS));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
