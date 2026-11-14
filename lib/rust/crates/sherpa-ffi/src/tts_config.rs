//! Exact configuration ABI from sherpa-onnx v2.0.1 (Apache-2.0).
//! Generated with scripts/sherpa-bindings.py; never omit unused trailing fields.
use std::ffi::c_char;

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineTtsVitsModelConfig {
    pub model: *const c_char,
    pub lexicon: *const c_char,
    pub tokens: *const c_char,
    pub data_dir: *const c_char,
    pub noise_scale: f32,
    pub noise_scale_w: f32,
    pub length_scale: f32,
    pub dict_dir: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineTtsMatchaModelConfig {
    pub acoustic_model: *const c_char,
    pub vocoder: *const c_char,
    pub lexicon: *const c_char,
    pub tokens: *const c_char,
    pub data_dir: *const c_char,
    pub noise_scale: f32,
    pub length_scale: f32,
    pub dict_dir: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineTtsKokoroModelConfig {
    pub model: *const c_char,
    pub voices: *const c_char,
    pub tokens: *const c_char,
    pub data_dir: *const c_char,
    pub length_scale: f32,
    pub dict_dir: *const c_char,
    pub lexicon: *const c_char,
    pub lang: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineTtsKittenModelConfig {
    pub model: *const c_char,
    pub voices: *const c_char,
    pub tokens: *const c_char,
    pub data_dir: *const c_char,
    pub length_scale: f32,
    pub lexicon: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineTtsZipvoiceModelConfig {
    pub tokens: *const c_char,
    pub encoder: *const c_char,
    pub decoder: *const c_char,
    pub vocoder: *const c_char,
    pub data_dir: *const c_char,
    pub lexicon: *const c_char,
    pub feat_scale: f32,
    pub t_shift: f32,
    pub target_rms: f32,
    pub guidance_scale: f32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineTtsPocketModelConfig {
    pub lm_flow: *const c_char,
    pub lm_main: *const c_char,
    pub encoder: *const c_char,
    pub decoder: *const c_char,
    pub text_conditioner: *const c_char,
    pub vocab_json: *const c_char,
    pub token_scores_json: *const c_char,
    pub voice_embedding_cache_capacity: i32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineTtsSupertonicModelConfig {
    pub duration_predictor: *const c_char,
    pub text_encoder: *const c_char,
    pub vector_estimator: *const c_char,
    pub vocoder: *const c_char,
    pub tts_json: *const c_char,
    pub unicode_indexer: *const c_char,
    pub voice_style: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineTtsModelConfig {
    pub vits: OfflineTtsVitsModelConfig,
    pub num_threads: i32,
    pub debug: i32,
    pub provider: *const c_char,
    pub matcha: OfflineTtsMatchaModelConfig,
    pub kokoro: OfflineTtsKokoroModelConfig,
    pub kitten: OfflineTtsKittenModelConfig,
    pub zipvoice: OfflineTtsZipvoiceModelConfig,
    pub pocket: OfflineTtsPocketModelConfig,
    pub supertonic: OfflineTtsSupertonicModelConfig,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineTtsConfig {
    pub model: OfflineTtsModelConfig,
    pub rule_fsts: *const c_char,
    pub max_num_sentences: i32,
    pub rule_fars: *const c_char,
    pub silence_scale: f32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct GenerationConfig {
    pub silence_scale: f32,
    pub speed: f32,
    pub sid: i32,
    pub reference_audio: *const f32,
    pub reference_audio_len: i32,
    pub reference_sample_rate: i32,
    pub reference_text: *const c_char,
    pub num_steps: i32,
    pub extra: *const c_char,
    pub phoneme_codepoints: *const i32,
    pub phoneme_codepoints_lens: *const i32,
    pub phoneme_codepoints_num_sentences: i32,
    pub tokens: *const *const c_char,
    pub tokens_lens: *const i32,
    pub tokens_num_sentences: i32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct GeneratedAudio {
    pub samples: *const f32,
    pub n: i32,
    pub sample_rate: i32,
}

#[cfg(all(test, target_pointer_width = "64"))]
mod tests;
