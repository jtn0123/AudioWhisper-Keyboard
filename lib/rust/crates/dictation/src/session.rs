//! Session decisions. Android executes the ordered effects after the native lock is released.
use crate::mic_key::{Action, MicKey, Status};

pub const KEEP_LOADED_MS: i64 = 300_000;
pub const KEEP_HIDDEN_MS: i64 = 60_000;

#[derive(Debug, Default)]
pub struct Session {
    pub generation: i32,
    status: i32,
    pub phase: &'static str,
    warming: bool,
    level: String,
    preview: Vec<u16>,
    released: bool,
    previewing: bool,
    model: u8,
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
        "failed"
    } else {
        "not_caught"
    }
}
fn text(s: impl ToString) -> Vec<u16> {
    s.to_string().encode_utf16().collect()
}

impl Session {
    pub fn speech(&mut self, id: i32, detected: bool) {
        if self.listening(id) {
            self.phase = if detected {
                "speech"
            } else {
                "trailing_silence"
            };
        }
    }
    pub fn model(&mut self, operation: &str) -> Result<i32, String> {
        match operation {
            "get" => Ok(i32::from(self.model)),
            "loaded" => {
                self.model = 1;
                Ok(0)
            }
            "failed" => {
                self.model = 2;
                Ok(0)
            }
            "retry" => {
                if self.model == 2 {
                    self.model = 0;
                }
                Ok(0)
            }
            "isLoaded" => Ok(i32::from(self.model == 1)),
            "close" => {
                let loaded = self.model == 1;
                if loaded {
                    self.model = 0;
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
            phase: "idle",
            level: "0.0".into(),
            ..Self::default()
        }
    }
    fn current(&self, id: i32) -> bool {
        id == self.generation && self.status != 0
    }
    fn listening(&self, id: i32) -> bool {
        id == self.generation && self.status == 1
    }
    fn schedule(&self, now: i64, effects: &mut Vec<Vec<u16>>) {
        effects.extend([
            text("schedule"),
            text(release_at(self.idle_since, now, self.shown)),
        ]);
    }
    fn idle(&mut self, now: i64, effects: &mut Vec<Vec<u16>>) {
        self.status = 0;
        self.level = "0.0".into();
        self.warming = false;
        self.preview.clear();
        self.idle_since = now;
        self.schedule(now, effects);
    }
    fn abandon(&mut self, now: i64, effects: &mut Vec<Vec<u16>>) {
        if self.status == 0 {
            return;
        }
        self.generation = self.generation.wrapping_add(1);
        effects.push(text("discard"));
        self.phase = "cancelled";
        self.idle(now, effects);
    }
    fn finish(&mut self, effects: &mut Vec<Vec<u16>>) {
        effects.push(text("flush"));
        self.status = 2;
        self.level = "0.0".into();
        self.phase = "flushing";
        effects.extend([text("barrier"), text(self.generation)]);
    }
    fn start(effects: &mut Vec<Vec<u16>>) {
        effects.push(text("check"));
    }
    fn toggle(&mut self, effects: &mut Vec<Vec<u16>>) {
        match self.status {
            0 => Self::start(effects),
            1 => self.finish(effects),
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
                "press" => {
                    let status = match self.status {
                        0 => Status::Idle,
                        1 => Status::Listening,
                        _ => Status::Finishing,
                    };
                    if self.mic.press(status, now) == Action::Start {
                        Self::start(&mut effects);
                    }
                }
                "releaseMic" => match self.mic.release(now) {
                    Action::Finish if self.status == 1 => self.finish(&mut effects),
                    Action::EndHold if self.status == 1 => {
                        effects.push(text("tick"));
                        self.finish(&mut effects);
                    }
                    Action::Toggle => self.toggle(&mut effects),
                    _ => {}
                },
                "ready" => {
                    let problem = if s(0)? != "true" {
                        "field_off"
                    } else if s(1)? != "true" {
                        "allow_mic"
                    } else {
                        match s(2)?.as_str() {
                            "ready" => "",
                            "missing" => "not_installed",
                            _ => "damaged",
                        }
                    };
                    if problem.is_empty() {
                        self.generation = self.generation.wrapping_add(1);
                        effects.extend([text("listen"), text(self.generation)]);
                    } else {
                        self.phase = "error";
                        effects.extend([text("problem"), text(problem)]);
                        if problem == "allow_mic" {
                            effects.push(text("appSettings"));
                        }
                        if problem == "not_installed" || problem == "damaged" {
                            effects.push(text("modelSettings"));
                        }
                    }
                }
                "opened" => {
                    self.typed = false;
                    self.failures = 0;
                    self.composition.reset();
                    self.explained = false;
                    self.status = 1;
                    self.phase = "listening";
                    effects.extend([text("preload"), text(self.generation)]);
                }
                "openError" => {
                    self.phase = "error";
                    effects.extend([text("problem"), text("mic_busy")]);
                }
                "stop" if self.status == 1 => {
                    self.explained = true;
                    self.finish(&mut effects);
                }
                "cancel" => self.abandon(now, &mut effects),
                "visible" => {
                    self.shown = s(0)? == "true";
                    if self.status == 0 && self.idle_since != 0 {
                        self.schedule(now, &mut effects);
                    }
                }
                "release" => {
                    self.abandon(now, &mut effects);
                    self.released = true;
                    effects.extend([text("removeUnload"), text("closeWorker")]);
                }
                "free" if self.status == 0 => {
                    effects.extend([text("removeUnload"), text("unload")])
                }
                "unload" => effects.push(text("unload")),
                "warming" if self.current(id) => {
                    self.warming = true;
                    self.phase = "warming";
                }
                "warmed" => {
                    self.warming = false;
                    if self.phase == "warming" {
                        self.phase = "listening";
                    }
                }
                "modelError" if self.current(id) => {
                    effects.extend([text("problem"), text("model_failed")]);
                    self.abandon(now, &mut effects);
                    self.phase = "error";
                }
                "micError" if self.listening(id) => {
                    effects.extend([text("problem"), text("mic_stopped")]);
                    self.explained = true;
                    self.finish(&mut effects);
                }
                "level" if self.listening(id) => self.level = s(0)?,
                "preview" if self.listening(id) => {
                    self.preview = chars::trim(fields.first().ok_or("missing preview")?).to_vec()
                }
                "vadOn" if self.listening(id) => self.phase = "speech",
                "vadOff" if self.listening(id) => self.phase = "trailing_silence",
                "finalizing" if self.current(id) => self.phase = "finalizing",
                "final" if id == self.generation => {
                    self.preview.clear();
                    if s(0)? == "true" {
                        let raw = fields.get(1).ok_or("missing result")?;
                        if !chars::trim(raw).is_empty() {
                            self.typed = true;
                            effects.extend([text("tidy"), raw.clone()]);
                        }
                    } else {
                        self.failures += 1;
                    }
                }
                "finished" if id == self.generation => {
                    if !self.explained {
                        let msg = ending(self.typed, self.failures);
                        if !msg.is_empty() {
                            effects.extend([text("problem"), text(msg)]);
                        }
                    }
                    self.phase = "committed";
                    self.idle(now, &mut effects);
                }
                "tidied" => effects.extend(
                    self.composition
                        .prepare(fields.first().ok_or("missing tidied phrase")?),
                ),
                "compose" => {
                    let before = fields.first().ok_or("missing editor context")?.clone();
                    if fields.len() > 1 {
                        self.composition.set_names(&fields[1..]);
                    }
                    effects.extend(self.composition.plan(before));
                }
                "snapshot" | "stop" | "free" | "warming" | "modelError" | "micError" | "level"
                | "preview" | "vadOn" | "vadOff" | "finalizing" | "final" | "finished" => {}
                _ => return Err("unknown dictation event".into()),
            }
        }
        let mut plan = vec![
            text(self.status),
            text(self.warming),
            text(&self.level),
            self.preview.clone(),
            text(self.generation),
            text(self.phase),
        ];
        plan.extend(effects);
        Ok(plan)
    }
}

#[cfg(test)]
mod tests;
