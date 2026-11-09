//! JNI entry points for GIPHY's URLs and answers (`gif::giphy`), called by `GiphyNative` in
//! lib/rust's Kotlin.

use std::ptr;

use gif::giphy::{self, Gif};
use jni_sys::{JNIEnv, jclass, jobjectArray, jstring};

use crate::jni::{self, illegal};

/// The GIFs in GIPHY's answer `body`: `[String[], float[]]`, the strings five a GIF (id, title,
/// preview URL, send URL, page URL) and the floats one (aspect ratio). Throws
/// IllegalArgumentException if the body isn't the JSON expected, as kotlinx.serialization did.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GiphyNative_parse(
    env: *mut JNIEnv,
    _: jclass,
    body: jstring,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let body = env.read_string(body, "body")?;
            let gifs = giphy::parse(&body).map_err(|_| illegal("GIPHY's answer is unreadable"))?;
            let strings: Vec<&[u16]> = gifs.iter().flat_map(texts).collect();
            let ratios: Vec<f32> = gifs.iter().map(|it| it.aspect_ratio).collect();
            env.make_objects(&[&|| env.make_strings(&strings), &|| env.make(&ratios)])
        })
    }
}

fn texts(gif: &Gif) -> [&[u16]; 5] {
    [
        &gif.id,
        &gif.title,
        &gif.preview_url,
        &gif.send_url,
        &gif.page_url,
    ]
}

/// The URL that asks `endpoint` with these params, `names[i]` being `values[i]`'s, then the key
/// and GIPHY's settings.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GiphyNative_url(
    env: *mut JNIEnv,
    _: jclass,
    api_key: jstring,
    endpoint: jstring,
    names: jobjectArray,
    values: jobjectArray,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let api_key = env.read_string(api_key, "apiKey")?;
            let endpoint = env.read_string(endpoint, "endpoint")?;
            let names = env.read_strings(names, "names")?;
            let values = env.read_strings(values, "values")?;
            if names.len() != values.len() {
                return Err(illegal(format!(
                    "{} names for {} values",
                    names.len(),
                    values.len()
                )));
            }
            let params = (names.into_iter().zip(values))
                .map(|pair| match pair {
                    (Some(name), Some(value)) => Ok((name, value)),
                    _ => Err(illegal("a null in names or values")),
                })
                .collect::<jni::Result<Vec<_>>>()?;
            env.make_string(&giphy::url(&api_key, &endpoint, &params))
        })
    }
}

/// The URL that asks for trending GIFs.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GiphyNative_trendingUrl(
    env: *mut JNIEnv,
    _: jclass,
    api_key: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make_string(&giphy::trending_url(&env.read_string(api_key, "apiKey")?))
        })
    }
}

/// The URL that searches for `query`, cut to GIPHY's longest search.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GiphyNative_searchUrl(
    env: *mut JNIEnv,
    _: jclass,
    api_key: jstring,
    query: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let api_key = env.read_string(api_key, "apiKey")?;
            let query = env.read_string(query, "query")?;
            env.make_string(&giphy::search_url(&api_key, &query))
        })
    }
}
