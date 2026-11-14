//! The sole sherpa native boundary. Safe owners hide C pointers, streams and snapshots.
mod api;
pub mod config;
mod tts;
mod tts_config;
mod vad;
use api::Api;
use config::{
    FeatureConfig, OfflineModelConfig, OfflineRecognizerConfig, OfflineTransducerModelConfig,
};
use std::{
    ffi::{CStr, CString, c_void},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
pub use tts::{SharedTts, Tts};
pub use vad::{Batch, Vad};

#[derive(Debug)]
struct Owned {
    api: Arc<Api>,
    ptr: *const c_void,
    destroy: unsafe extern "C" fn(*const c_void),
}
// SAFETY: CPU sherpa objects have no creation-thread affinity. The private owners move only
// between exclusive borrows (VAD/streams) or behind the recognizer mutex. Drop cannot overlap
// a C call, and each owner retains its library until destruction has finished.
unsafe impl Send for Owned {}
impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: this owner is unique. Streams/results are dropped before their recognizer.
        unsafe { (self.destroy)(self.ptr) }
    }
}
impl Owned {
    fn new(
        api: Arc<Api>,
        ptr: *const c_void,
        destroy: unsafe extern "C" fn(*const c_void),
    ) -> Result<Self, String> {
        if ptr.is_null() {
            Err("sherpa could not allocate model or stream".into())
        } else {
            Ok(Self { api, ptr, destroy })
        }
    }
}
fn cstring(s: &str) -> Result<CString, String> {
    CString::new(s).map_err(|_| "NUL in model path".into())
}
fn count(samples: &[f32]) -> Result<i32, String> {
    i32::try_from(samples.len()).map_err(|_| "PCM buffer too large".into())
}
fn load() -> Result<Arc<Api>, String> {
    let api = Api::load()?;
    // SAFETY: version is a static, NUL-terminated string owned by the loaded library.
    let version = unsafe { CStr::from_ptr((api.version)()) };
    if version.to_bytes() != b"2.0.1" {
        return Err("Expected sherpa-onnx 2.0.1".into());
    }
    Ok(api)
}

/// Decode is serialized; cancel is atomic and can invalidate an in-flight decode. Closing waits
/// for the synchronous C decode and stream destruction before the recognizer can be freed.
#[derive(Debug)]
pub struct Recognizer {
    owner: Mutex<Owned>,
    generation: AtomicU64,
}
impl Recognizer {
    pub fn new(dir: &str) -> Result<Self, String> {
        let api = load()?;
        let paths = [
            "encoder.int8.onnx",
            "decoder.int8.onnx",
            "joiner.int8.onnx",
            "tokens.txt",
        ]
        .map(|name| cstring(&format!("{dir}/{name}")));
        let [encoder, decoder, joiner, tokens] = paths;
        let (encoder, decoder, joiner, tokens) = (encoder?, decoder?, joiner?, tokens?);
        let config = OfflineRecognizerConfig {
            feat_config: FeatureConfig {
                sample_rate: dictation::segmentation::SAMPLE_RATE as i32,
                feature_dim: 80,
            },
            model_config: OfflineModelConfig {
                transducer: OfflineTransducerModelConfig {
                    encoder: encoder.as_ptr(),
                    decoder: decoder.as_ptr(),
                    joiner: joiner.as_ptr(),
                },
                tokens: tokens.as_ptr(),
                num_threads: 4,
                provider: c"cpu".as_ptr(),
                model_type: c"nemo_transducer".as_ptr(),
                ..Default::default()
            },
            decoding_method: c"greedy_search".as_ptr(),
            max_active_paths: 4,
            hotwords_score: 1.5,
            ..Default::default()
        };
        // SAFETY: every config field has the exact C layout; paths live through synchronous creation.
        let ptr = unsafe { (api.create_recognizer)(&config) };
        let destroy = api.destroy_recognizer;
        Ok(Self {
            owner: Mutex::new(Owned::new(api, ptr, destroy)?),
            generation: AtomicU64::new(0),
        })
    }
    pub fn cancel(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
    }
    pub fn transcribe(&self, samples: &[f32]) -> Result<Vec<u16>, String> {
        let generation = self.generation.load(Ordering::Acquire);
        let n = count(samples)?;
        let owner = self
            .owner
            .lock()
            .map_err(|_| "ASR owner poisoned".to_owned())?;
        if self.generation.load(Ordering::Acquire) != generation {
            return Err("ASR cancelled".into());
        }
        let api = &owner.api;
        // SAFETY: recognizer is live and locked; stream creation is synchronous.
        let stream = Owned::new(
            api.clone(),
            unsafe { (api.create_stream)(owner.ptr) },
            api.destroy_stream,
        )?;
        // Offline Parakeet has no InputFinished API and no caller-added tail padding. The
        // original JNI does acceptWaveform -> decode -> result; feature padding is internal.
        // SAFETY: samples remain valid for this synchronous call; both owners are live.
        unsafe {
            (api.accept)(
                stream.ptr,
                dictation::segmentation::SAMPLE_RATE as i32,
                samples.as_ptr(),
                n,
            );
            (api.decode)(owner.ptr, stream.ptr);
        }
        // SAFETY: decode completed; the stream is live until after its result is destroyed.
        let result = unsafe { (api.result)(stream.ptr) };
        if result.is_null() {
            return Err("No ASR result".into());
        }
        // SAFETY: result.text is NUL-terminated by sherpa and valid until destroy_result below.
        let text = unsafe { CStr::from_ptr((*result).text) }
            .to_str()
            .map(|s| s.encode_utf16().collect::<Vec<_>>())
            .map_err(|_| "Invalid ASR UTF-8".to_owned());
        // SAFETY: the unique result has been copied and is freed exactly once.
        unsafe { (api.destroy_result)(result) };
        if self.generation.load(Ordering::Acquire) != generation {
            return Err("ASR cancelled".into());
        }
        text.map(|text| dictation::normalize_result(&text))
    }
}
