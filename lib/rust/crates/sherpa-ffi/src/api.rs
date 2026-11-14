//! Resolved symbols stay alive as long as any owned recognizer, stream or VAD.
use crate::config::{OfflineRecognizerConfig, VadModelConfig};
use crate::tts_config::{GeneratedAudio, GenerationConfig, OfflineTtsConfig};
use libloading::Library;
use std::{
    ffi::{c_char, c_void},
    sync::Arc,
};
#[repr(C)]
#[derive(Debug)]
pub struct Segment {
    pub start: i64,
    pub samples: *const f32,
    pub n: i32,
}
// Only the first field is accessed. The library owns the complete result layout.
#[repr(C)]
#[derive(Debug)]
pub struct ResultText {
    pub text: *const c_char,
}

macro_rules! symbols {
    ($($field:ident : $ty:ty = $symbol:literal),+ $(,)?) => {
        #[derive(Debug)]
        pub struct Api { $(pub $field:$ty,)+ _library:Library }
        impl Api {
            pub fn load()->Result<Arc<Self>,String> {
                // SAFETY: the caller loads the pinned, packaged sherpa library, not arbitrary input.
                let library=unsafe {Library::new("libsherpa-onnx-c-api.so")}.map_err(|_|"Could not open sherpa C API".to_owned())?;
                $(// SAFETY: each signature is the exact v2.0.1 declaration. Api retains the library.
                let $field=unsafe {*library.get::<$ty>(concat!($symbol,"\0").as_bytes()).map_err(|_|concat!("Missing ",$symbol).to_owned())?};)+
                Ok(Arc::new(Self{$($field,)+_library:library}))
            }
        }
    }
}
symbols! {
    create_tts: unsafe extern "C" fn(*const OfflineTtsConfig)->*const c_void = "SherpaOnnxCreateOfflineTts",
    destroy_tts: unsafe extern "C" fn(*const c_void) = "SherpaOnnxDestroyOfflineTts",
    tts_sample_rate: unsafe extern "C" fn(*const c_void)->i32 = "SherpaOnnxOfflineTtsSampleRate",
    tts_generate: unsafe extern "C" fn(*const c_void,*const c_char,*const GenerationConfig,Option<unsafe extern "C" fn(*const f32,i32,f32,*mut c_void)->i32>,*mut c_void)->*const GeneratedAudio = "SherpaOnnxOfflineTtsGenerateWithConfig",
    destroy_audio: unsafe extern "C" fn(*const GeneratedAudio) = "SherpaOnnxDestroyOfflineTtsGeneratedAudio",
    version: unsafe extern "C" fn()->*const c_char = "SherpaOnnxGetVersionStr",
    create_recognizer: unsafe extern "C" fn(*const OfflineRecognizerConfig)->*const c_void = "SherpaOnnxCreateOfflineRecognizer",
    destroy_recognizer: unsafe extern "C" fn(*const c_void) = "SherpaOnnxDestroyOfflineRecognizer",
    create_stream: unsafe extern "C" fn(*const c_void)->*const c_void = "SherpaOnnxCreateOfflineStream",
    destroy_stream: unsafe extern "C" fn(*const c_void) = "SherpaOnnxDestroyOfflineStream",
    accept: unsafe extern "C" fn(*const c_void,i32,*const f32,i32) = "SherpaOnnxAcceptWaveformOffline",
    decode: unsafe extern "C" fn(*const c_void,*const c_void) = "SherpaOnnxDecodeOfflineStream",
    result: unsafe extern "C" fn(*const c_void)->*const ResultText = "SherpaOnnxGetOfflineStreamResult",
    destroy_result: unsafe extern "C" fn(*const ResultText) = "SherpaOnnxDestroyOfflineRecognizerResult",
    create_vad: unsafe extern "C" fn(*const VadModelConfig,f32)->*const c_void = "SherpaOnnxCreateVoiceActivityDetector",
    destroy_vad: unsafe extern "C" fn(*const c_void) = "SherpaOnnxDestroyVoiceActivityDetector",
    vad_accept: unsafe extern "C" fn(*const c_void,*const f32,i32) = "SherpaOnnxVoiceActivityDetectorAcceptWaveform",
    vad_empty: unsafe extern "C" fn(*const c_void)->i32 = "SherpaOnnxVoiceActivityDetectorEmpty",
    vad_detected: unsafe extern "C" fn(*const c_void)->i32 = "SherpaOnnxVoiceActivityDetectorDetected",
    vad_front: unsafe extern "C" fn(*const c_void)->*const Segment = "SherpaOnnxVoiceActivityDetectorFront",
    destroy_segment: unsafe extern "C" fn(*const Segment) = "SherpaOnnxDestroySpeechSegment",
    vad_pop: unsafe extern "C" fn(*const c_void) = "SherpaOnnxVoiceActivityDetectorPop",
    vad_flush: unsafe extern "C" fn(*const c_void) = "SherpaOnnxVoiceActivityDetectorFlush",
}
