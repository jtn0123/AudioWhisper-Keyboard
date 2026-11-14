//! Audio history, VAD lead-in and bounded preview schedule from PhraseListener/AudioHistory.
pub const SAMPLE_RATE: usize = 16_000;
pub const CHUNK_SAMPLES: usize = SAMPLE_RATE / 10;
pub const VAD_THRESHOLD: f32 = 0.5;
pub const MIN_SILENCE_SECONDS: f32 = 0.6;
pub const MIN_SPEECH_SECONDS: f32 = 0.25;
pub const MAX_SPEECH_SECONDS: f32 = 20.0;
pub const VAD_WINDOW: i32 = 512;
pub const LEAD_IN: i64 = (SAMPLE_RATE * 3 / 10) as i64;
pub const DETECTION_DELAY: i64 =
    (SAMPLE_RATE as f32 * MIN_SPEECH_SECONDS) as i64 + VAD_WINDOW as i64;
pub const PREVIEW_EVERY: i64 = SAMPLE_RATE as i64;
pub const PREVIEW_MAX: i64 = (SAMPLE_RATE * 10) as i64;
pub const HISTORY: usize = SAMPLE_RATE * (MAX_SPEECH_SECONDS as usize + 5);

#[derive(Debug)]
pub struct Segmentation {
    ring: Vec<f32>,
    pub end: i64,
    speech_from: Option<i64>,
    next_preview: i64,
    last_end: i64,
}
impl Default for Segmentation {
    fn default() -> Self {
        Self {
            ring: vec![0.0; HISTORY],
            end: 0,
            speech_from: None,
            next_preview: 0,
            last_end: 0,
        }
    }
}
impl Segmentation {
    pub fn add(&mut self, audio: &[f32]) {
        for &s in audio {
            self.ring[self.end as usize % HISTORY] = s;
            self.end += 1;
        }
    }
    pub fn slice(&self, from: i64, until: i64) -> Vec<f32> {
        let first = from.max(self.end - HISTORY as i64).max(0);
        let last = until.min(self.end);
        (first..last)
            .map(|i| self.ring[i as usize % HISTORY])
            .collect()
    }
    pub fn phrase(&mut self, start: i64, samples: &[f32]) -> Vec<f32> {
        let mut audio = self.slice(start - LEAD_IN, start);
        audio.extend_from_slice(samples);
        self.last_end = start + samples.len() as i64;
        self.speech_from = None;
        audio
    }
    pub fn preview(&mut self, detected: bool) -> Option<Vec<f32>> {
        if !detected {
            self.speech_from = None;
            return None;
        }
        let now = self.end;
        let from = *self.speech_from.get_or_insert_with(|| {
            self.next_preview = now + PREVIEW_EVERY / 2;
            self.last_end.max(now - DETECTION_DELAY - LEAD_IN)
        });
        if now < self.next_preview {
            return None;
        }
        self.next_preview = now + PREVIEW_EVERY;
        Some(self.slice(from.max(now - PREVIEW_MAX), now))
    }
}
