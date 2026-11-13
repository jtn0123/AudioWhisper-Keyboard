//! Clipboard policy JNI transport, with lossless UTF-16 text equality and MIME matching.
use crate::jni::{self, Env, Result, illegal};
use jni_sys::{JNIEnv, jboolean, jclass, jlongArray, jobjectArray, jstring};
use render::clipboard_policy::{self as policy, Clip};
use std::ptr;
fn texts(env: &Env, input: jobjectArray) -> Result<Vec<Vec<u16>>> {
    env.read_strings(input, "clipboard strings")?
        .into_iter()
        .map(|s| s.ok_or_else(|| illegal("null clipboard string")))
        .collect()
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ClipboardNative_plan(
    env: *mut JNIEnv,
    _: jclass,
    records: jlongArray,
    strings: jobjectArray,
    config: jlongArray,
) -> jobjectArray {
    // SAFETY: all JVM arrays are copied during this call; the output contains only local arrays.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let data = env.read::<i64>(records, "clipboard records")?;
            let strings = texts(env, strings)?;
            let c = env.read::<i64>(config, "clipboard config")?;
            if data.len() % 5 != 0 || strings.len() != data.len() / 5 + 1 || c.len() != 9 {
                return Err(illegal("clipboard payload"));
            }
            let clips: Vec<_> = data
                .as_chunks::<5>()
                .0
                .iter()
                .enumerate()
                .map(|(i, d)| Clip {
                    timestamp: d[0],
                    kind: d[1] as i32,
                    pinned: d[2] != 0,
                    sensitive: d[3] != 0,
                    text: (d[4] != 0).then(|| strings[i].clone()),
                })
                .collect();
            let g = policy::groups(&clips, c[0]);
            let dup = policy::duplicate(
                &clips,
                (c[8] != 0).then(|| strings.last().unwrap().as_slice()),
                c[1] != 0,
            );
            let out: Vec<Vec<i32>> = vec![
                policy::order(&clips),
                g[0].clone(),
                g[1].clone(),
                g[2].clone(),
                policy::limit(&clips, c[0], c[2] != 0, c[3] as i32),
                policy::expiry(&clips, c[0], c[4] != 0, c[5] as i32, c[6] != 0, c[7] as i32),
            ]
            .into_iter()
            .map(|v| v.into_iter().map(|i| i as i32).collect())
            .chain(std::iter::once(
                dup.map_or(vec![-1, 0], |(i, p)| vec![i as i32, i32::from(p)]),
            ))
            .collect();
            let makers: Vec<Box<dyn Fn() -> Result<jni_sys::jobject> + '_>> = out
                .iter()
                .map(|a| {
                    Box::new(|| env.make(a).map(|v| v as jni_sys::jobject))
                        as Box<dyn Fn() -> Result<jni_sys::jobject>>
                })
                .collect();
            env.make_objects(&makers.iter().map(|f| f.as_ref()).collect::<Vec<_>>())
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ClipboardRules_mimeMatches(
    env: *mut JNIEnv,
    _: jclass,
    concrete: jstring,
    desired: jstring,
) -> jboolean {
    // SAFETY: the JVM owns the strings for the call; nothing borrowed escapes.
    unsafe {
        jni::run(env, 0, |env| {
            Ok(u8::from(policy::mime_matches(
                &env.read_string(concrete, "concrete")?,
                &env.read_string(desired, "desired")?,
            )))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ClipboardRules_eligible(
    env: *mut JNIEnv,
    _: jclass,
    types: jobjectArray,
    editor: jobjectArray,
) -> jboolean {
    // SAFETY: input object arrays and their strings are copied before the pure policy runs.
    unsafe {
        jni::run(env, 0, |env| {
            Ok(u8::from(policy::eligible(
                &texts(env, types)?,
                &texts(env, editor)?,
            )))
        })
    }
}
