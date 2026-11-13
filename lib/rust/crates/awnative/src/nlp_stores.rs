//! Bulk store transport; callers hold session monitors and keep immutable snapshots reachable.
use crate::{
    handle,
    jni::{self, Error},
};
use jni_sys::{JNIEnv, jboolean, jclass, jlong, jobjectArray, jstring};
use std::ptr;
use suggest::{personal_words::PersonalWords, session::Session};

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_NlpNative_isName(
    env: *mut JNIEnv,
    _: jclass,
    dictionary: jlong,
    personal: jlong,
    word: jstring,
) -> jboolean {
    // SAFETY: the session wrapper serializes calls/close; reachability fences pin the personal snapshot.
    unsafe {
        jni::run(env, 0, |env| {
            let dictionary = if dictionary == 0 {
                None
            } else {
                Some(handle::locked::<Session>(dictionary, "session")?)
            };
            let personal = handle::object::<PersonalWords>(personal, "personal words")?;
            Ok(u8::from(nlp::stores::is_name(
                &env.read_string(word, "word")?,
                personal,
                dictionary.as_deref(),
            )))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_NlpNative_unknownWords(
    env: *mut JNIEnv,
    _: jclass,
    dictionary: jlong,
    words: jobjectArray,
) -> jobjectArray {
    // SAFETY: the wrapper keeps its session live and locked against close; JVM strings are copied.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let words = env
                .read_strings(words, "words")?
                .into_iter()
                .map(|v| v.ok_or(Error::NullPointer("word")))
                .collect::<jni::Result<Vec<_>>>()?;
            let dictionary = handle::locked::<Session>(dictionary, "session")?;
            env.make_strings(&nlp::stores::unknown_words(&dictionary, words))
        })
    }
}
