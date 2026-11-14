use crate::{
    Owned,
    config::{SileroVadModelConfig, VadModelConfig},
    count, cstring, load,
};
use dictation::segmentation::{self, Segmentation};
use std::slice;

#[derive(Debug)]
pub struct SegmentAudio {
    pub start: i64,
    pub samples: Vec<f32>,
    pub raw_len: usize,
}
#[derive(Debug)]
pub struct Batch {
    pub detected: bool,
    pub transition: i32,
    pub end: i64,
    pub segments: Vec<SegmentAudio>,
    pub preview: Option<Vec<f32>>,
}
#[derive(Debug)]
pub struct Vad {
    owner: Option<Owned>,
    history: Segmentation,
    detected: bool,
}
impl Vad {
    pub fn new(model: &str) -> Result<Self, String> {
        let api = load()?;
        let model = cstring(model)?;
        let config = VadModelConfig {
            silero_vad: SileroVadModelConfig {
                model: model.as_ptr(),
                threshold: segmentation::VAD_THRESHOLD,
                min_silence_duration: segmentation::MIN_SILENCE_SECONDS,
                min_speech_duration: segmentation::MIN_SPEECH_SECONDS,
                window_size: segmentation::VAD_WINDOW,
                max_speech_duration: segmentation::MAX_SPEECH_SECONDS,
            },
            sample_rate: dictation::segmentation::SAMPLE_RATE as i32,
            num_threads: 1,
            provider: c"cpu".as_ptr(),
            ..Default::default()
        };
        // SAFETY: exact v2.0.1 layout, path valid until synchronous creation returns. JNI uses 60 s.
        let ptr = unsafe { (api.create_vad)(&config, 60.0) };
        let destroy = api.destroy_vad;
        Ok(Self {
            owner: Some(Owned::new(api, ptr, destroy)?),
            history: Segmentation::default(),
            detected: false,
        })
    }
    pub fn push(&mut self, samples: &[f32]) -> Result<Batch, String> {
        let n = count(samples)?;
        let owner = self.owner.as_ref().ok_or("VAD stopped")?;
        self.history.add(samples);
        // SAFETY: live unique VAD and PCM valid throughout this synchronous accept.
        unsafe { (owner.api.vad_accept)(owner.ptr, samples.as_ptr(), n) };
        self.drain(true)
    }
    /// The executor has joined the producer. Flush and snapshot destruction finish before free.
    pub fn stop(&mut self, keep: bool) -> Result<Batch, String> {
        let owner = self.owner.as_ref().ok_or("VAD stopped")?;
        if keep {
            // SAFETY: producer stopped; this is the unique live detector.
            unsafe { (owner.api.vad_flush)(owner.ptr) };
        }
        let result = self.drain(false);
        self.owner.take();
        result.map(|mut b| {
            if !keep {
                b.segments.clear();
            }
            b
        })
    }
    fn drain(&mut self, preview: bool) -> Result<Batch, String> {
        let owner = self.owner.as_ref().ok_or("VAD stopped")?;
        let api = &owner.api;
        let mut segments = Vec::new();
        // SAFETY: owner is live and uniquely borrowed through this entire drain.
        while unsafe { (api.vad_empty)(owner.ptr) } == 0 {
            // SAFETY: a nonempty queue supplies an allocated snapshot owned by this call.
            let segment = unsafe { (api.vad_front)(owner.ptr) };
            if segment.is_null() {
                return Err("No VAD segment".into());
            }
            // SAFETY: sherpa owns the snapshot with n initialized floats until destroyed below.
            let (start, n, samples) = unsafe {
                let n = (*segment).n;
                if n < 0 {
                    (0, 0, Vec::new())
                } else {
                    let data = if n == 0 {
                        &[][..]
                    } else {
                        slice::from_raw_parts((*segment).samples, n as usize)
                    };
                    (
                        (*segment).start,
                        n,
                        self.history.phrase((*segment).start, data),
                    )
                }
            };
            // SAFETY: copied snapshot is destroyed once, then its source queue entry is popped.
            unsafe {
                (api.destroy_segment)(segment);
                (api.vad_pop)(owner.ptr);
            }
            segments.push(SegmentAudio {
                start,
                samples,
                raw_len: n as usize,
            });
        }
        // SAFETY: owner remains live and no producer accesses it concurrently.
        let detected = unsafe { (api.vad_detected)(owner.ptr) } != 0;
        let transition = if detected == self.detected {
            0
        } else if detected {
            1
        } else {
            -1
        };
        self.detected = detected;
        Ok(Batch {
            detected,
            transition,
            end: self.history.end,
            segments,
            preview: if preview {
                self.history.preview(detected)
            } else {
                None
            },
        })
    }
}
