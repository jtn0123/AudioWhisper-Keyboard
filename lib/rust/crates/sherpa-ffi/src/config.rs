//! Exact configuration ABI from sherpa-onnx v2.0.1 (Apache-2.0).
//! Generated with scripts/sherpa-bindings.py; never omit unused trailing fields.
use std::ffi::c_char;

#[repr(C)]
#[derive(Debug, Default)]
pub struct FeatureConfig {
    pub sample_rate: i32,
    pub feature_dim: i32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineTransducerModelConfig {
    pub encoder: *const c_char,
    pub decoder: *const c_char,
    pub joiner: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineParaformerModelConfig {
    pub model: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineNemoEncDecCtcModelConfig {
    pub model: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineWhisperModelConfig {
    pub encoder: *const c_char,
    pub decoder: *const c_char,
    pub language: *const c_char,
    pub task: *const c_char,
    pub tail_paddings: i32,
    pub enable_token_timestamps: i32,
    pub enable_segment_timestamps: i32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineTdnnModelConfig {
    pub model: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineSenseVoiceModelConfig {
    pub model: *const c_char,
    pub language: *const c_char,
    pub use_itn: i32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineMoonshineModelConfig {
    pub preprocessor: *const c_char,
    pub encoder: *const c_char,
    pub uncached_decoder: *const c_char,
    pub cached_decoder: *const c_char,
    pub merged_decoder: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineFireRedAsrModelConfig {
    pub encoder: *const c_char,
    pub decoder: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineDolphinModelConfig {
    pub model: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineZipformerCtcModelConfig {
    pub model: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineCanaryModelConfig {
    pub encoder: *const c_char,
    pub decoder: *const c_char,
    pub src_lang: *const c_char,
    pub tgt_lang: *const c_char,
    pub use_pnc: i32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineWenetCtcModelConfig {
    pub model: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineOmnilingualAsrCtcModelConfig {
    pub model: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineMedAsrCtcModelConfig {
    pub model: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineFunASRNanoModelConfig {
    pub encoder_adaptor: *const c_char,
    pub llm: *const c_char,
    pub embedding: *const c_char,
    pub tokenizer: *const c_char,
    pub system_prompt: *const c_char,
    pub user_prompt: *const c_char,
    pub max_new_tokens: i32,
    pub temperature: f32,
    pub top_p: f32,
    pub seed: i32,
    pub language: *const c_char,
    pub itn: i32,
    pub hotwords: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineFireRedAsrCtcModelConfig {
    pub model: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineQwen3ASRModelConfig {
    pub conv_frontend: *const c_char,
    pub encoder: *const c_char,
    pub decoder: *const c_char,
    pub tokenizer: *const c_char,
    pub max_total_len: i32,
    pub max_new_tokens: i32,
    pub temperature: f32,
    pub top_p: f32,
    pub seed: i32,
    pub hotwords: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineCohereTranscribeModelConfig {
    pub encoder: *const c_char,
    pub decoder: *const c_char,
    pub language: *const c_char,
    pub use_punct: i32,
    pub use_itn: i32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineModelConfig {
    pub transducer: OfflineTransducerModelConfig,
    pub paraformer: OfflineParaformerModelConfig,
    pub nemo_ctc: OfflineNemoEncDecCtcModelConfig,
    pub whisper: OfflineWhisperModelConfig,
    pub tdnn: OfflineTdnnModelConfig,
    pub tokens: *const c_char,
    pub num_threads: i32,
    pub debug: i32,
    pub provider: *const c_char,
    pub model_type: *const c_char,
    pub modeling_unit: *const c_char,
    pub bpe_vocab: *const c_char,
    pub telespeech_ctc: *const c_char,
    pub sense_voice: OfflineSenseVoiceModelConfig,
    pub moonshine: OfflineMoonshineModelConfig,
    pub fire_red_asr: OfflineFireRedAsrModelConfig,
    pub dolphin: OfflineDolphinModelConfig,
    pub zipformer_ctc: OfflineZipformerCtcModelConfig,
    pub canary: OfflineCanaryModelConfig,
    pub wenet_ctc: OfflineWenetCtcModelConfig,
    pub omnilingual: OfflineOmnilingualAsrCtcModelConfig,
    pub medasr: OfflineMedAsrCtcModelConfig,
    pub funasr_nano: OfflineFunASRNanoModelConfig,
    pub fire_red_asr_ctc: OfflineFireRedAsrCtcModelConfig,
    pub qwen3_asr: OfflineQwen3ASRModelConfig,
    pub cohere_transcribe: OfflineCohereTranscribeModelConfig,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineLMConfig {
    pub model: *const c_char,
    pub scale: f32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct HomophoneReplacerConfig {
    pub dict_dir: *const c_char,
    pub lexicon: *const c_char,
    pub rule_fsts: *const c_char,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineCtcFstDecoderConfig {
    pub graph: *const c_char,
    pub max_active: i32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct OfflineRecognizerConfig {
    pub feat_config: FeatureConfig,
    pub model_config: OfflineModelConfig,
    pub lm_config: OfflineLMConfig,
    pub decoding_method: *const c_char,
    pub max_active_paths: i32,
    pub hotwords_file: *const c_char,
    pub hotwords_score: f32,
    pub rule_fsts: *const c_char,
    pub rule_fars: *const c_char,
    pub blank_penalty: f32,
    pub hr: HomophoneReplacerConfig,
    pub ctc_fst_decoder_config: OfflineCtcFstDecoderConfig,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct SileroVadModelConfig {
    pub model: *const c_char,
    pub threshold: f32,
    pub min_silence_duration: f32,
    pub min_speech_duration: f32,
    pub window_size: i32,
    pub max_speech_duration: f32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct TenVadModelConfig {
    pub model: *const c_char,
    pub threshold: f32,
    pub min_silence_duration: f32,
    pub min_speech_duration: f32,
    pub window_size: i32,
    pub max_speech_duration: f32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct VadModelConfig {
    pub silero_vad: SileroVadModelConfig,
    pub sample_rate: i32,
    pub num_threads: i32,
    pub provider: *const c_char,
    pub debug: i32,
    pub ten_vad: TenVadModelConfig,
}

#[cfg(all(test, target_pointer_width = "64"))]
mod tests;
