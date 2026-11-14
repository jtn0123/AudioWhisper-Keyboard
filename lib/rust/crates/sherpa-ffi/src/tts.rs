//! Synchronous Pocket generator. Cancellation is atomic; the active call owns the model.
//! No mutex is held across the audio callback. Close requests stop before dropping anything.
use crate::tts_config::*;
use crate::{Owned, count, cstring, load};
use reader::generator::{Generator, Lease};
use std::{ffi::c_void, sync::Arc};
#[derive(Debug)]
pub struct Tts {
    generator: Generator<Owned>,
    reference: Vec<f32>,
    reference_rate: i32,
    sample_rate: i32,
}
impl Tts {
    pub fn new(dir: &str, reference: Vec<f32>, reference_rate: i32) -> Result<Self, String> {
        count(&reference)?;
        if reference.is_empty() || reference_rate <= 0 {
            return Err("Invalid reference PCM".into());
        }
        let api = load()?;
        let paths = [
            "lm_flow.int8.onnx",
            "lm_main.int8.onnx",
            "encoder.onnx",
            "decoder.int8.onnx",
            "text_conditioner.onnx",
            "vocab.json",
            "token_scores.json",
        ]
        .map(|name| cstring(&format!("{dir}/{name}")))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
        let config = OfflineTtsConfig {
            model: OfflineTtsModelConfig {
                num_threads: 4,
                provider: c"cpu".as_ptr(),
                pocket: OfflineTtsPocketModelConfig {
                    lm_flow: paths[0].as_ptr(),
                    lm_main: paths[1].as_ptr(),
                    encoder: paths[2].as_ptr(),
                    decoder: paths[3].as_ptr(),
                    text_conditioner: paths[4].as_ptr(),
                    vocab_json: paths[5].as_ptr(),
                    token_scores_json: paths[6].as_ptr(),
                    voice_embedding_cache_capacity: 50,
                },
                ..Default::default()
            },
            max_num_sentences: 1,
            silence_scale: 0.2,
            ..Default::default()
        };
        // SAFETY: exact pinned ABI; all strings live until synchronous creation completes.
        let ptr = unsafe { (api.create_tts)(&config) };
        let destroy = api.destroy_tts;
        let owner = Owned::new(api, ptr, destroy)?;
        // SAFETY: the newly allocated generator is live and exclusively owned.
        let sample_rate = unsafe { (owner.api.tts_sample_rate)(owner.ptr) };
        Ok(Self {
            generator: Generator::new(owner),
            reference,
            reference_rate,
            sample_rate,
        })
    }
    pub fn sample_rate(&self) -> i32 {
        self.sample_rate
    }
    pub fn cancel(&self) {
        self.generator.cancel();
    }
    pub fn close(&self) {
        self.generator.close();
    }
    pub fn generate(&self, text: &str, audio: impl FnMut(&[f32]) -> bool) -> Result<(), String> {
        self.generate_inner(text, audio, None)
    }
    /// The comparison APK pins stochastic synthesis; normal builds omit this entry point.
    #[cfg(feature = "tts-fixture")]
    pub fn generate_fixture(
        &self,
        text: &str,
        seed: i32,
        audio: impl FnMut(&[f32]) -> bool,
    ) -> Result<(), String> {
        self.generate_inner(text, audio, Some(seed))
    }
    fn generate_inner(
        &self,
        text: &str,
        mut audio: impl FnMut(&[f32]) -> bool,
        seed: Option<i32>,
    ) -> Result<(), String> {
        let lease = self.generator.lease()?;
        let owner = lease.owner();
        let text = cstring(text)?;
        let extra = generation_extra(seed)?;
        let config = GenerationConfig {
            silence_scale: 0.2,
            speed: 1.0,
            sid: 0,
            reference_audio: self.reference.as_ptr(),
            reference_audio_len: count(&self.reference)?,
            reference_sample_rate: self.reference_rate,
            num_steps: 5,
            extra: extra
                .as_ref()
                .map_or(std::ptr::null(), |value| value.as_ptr()),
            ..Default::default()
        };
        let mut callback = Callback {
            lease: &lease,
            audio: &mut audio,
            failed: false,
        };
        // SAFETY: the lease owns the generator; text/config/reference/callback live through this
        // synchronous call. sherpa invokes the callback on this thread and never retains arg.
        let result = unsafe {
            (owner.api.tts_generate)(
                owner.ptr,
                text.as_ptr(),
                &config,
                Some(deliver),
                (&mut callback as *mut Callback<'_>).cast(),
            )
        };
        if result.is_null() {
            return Err("No TTS output".into());
        }
        // SAFETY: output is uniquely owned and every borrowed audio callback has returned.
        unsafe { (owner.api.destroy_audio)(result) };
        if callback.failed {
            Err("Invalid TTS callback".into())
        } else {
            Ok(())
        }
    }
}
struct Callback<'a> {
    lease: &'a Lease<'a, Owned>,
    audio: &'a mut dyn FnMut(&[f32]) -> bool,
    failed: bool,
}
unsafe extern "C" fn deliver(samples: *const f32, n: i32, _: f32, arg: *mut c_void) -> i32 {
    // SAFETY: generate passes a live Callback for this synchronous C invocation only.
    let callback = unsafe { &mut *arg.cast::<Callback<'_>>() };
    if !callback.lease.current() {
        return 0;
    }
    if n < 0 || (n > 0 && samples.is_null()) {
        callback.failed = true;
        return 0;
    }
    // SAFETY: sherpa supplies n valid samples for the duration of this callback. Empty buffers
    // use an empty slice so from_raw_parts never receives null, even with length zero.
    let samples = if n == 0 {
        &[]
    } else {
        // SAFETY: the non-null C buffer contains n samples until this callback returns.
        unsafe { std::slice::from_raw_parts(samples, n as usize) }
    };
    // Do not unwind into C even if a consumer panics.
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (callback.audio)(samples))) {
        Ok(keep) => i32::from(keep && callback.lease.current()),
        Err(_) => {
            callback.failed = true;
            0
        }
    }
}
fn generation_extra(seed: Option<i32>) -> Result<Option<std::ffi::CString>, String> {
    seed.map(|value| cstring(&format!("{{\"seed\":{value}}}")))
        .transpose()
}

// Keep public ownership explicit: callers share Tts, never C pointers or borrowed output.
pub type SharedTts = Arc<Tts>;

#[cfg(test)]
mod tests {
    #[test]
    fn comparison_seed_is_explicit_and_normal_generation_has_no_override() {
        assert!(super::generation_extra(None).unwrap().is_none());
        for (seed, expected) in [
            (42, br#"{"seed":42}"#.as_slice()),
            (-1, br#"{"seed":-1}"#.as_slice()),
            (0, br#"{"seed":0}"#.as_slice()),
        ] {
            assert_eq!(
                super::generation_extra(Some(seed))
                    .unwrap()
                    .unwrap()
                    .as_bytes(),
                expected
            );
        }
    }
}
