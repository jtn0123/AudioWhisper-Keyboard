//! JNI entry points for emoji data and suggestions (the `emoji` crate), called by `EmojiNative`
//! in lib/rust's Kotlin.
//!
//! An emoji file is parsed once into a read-only handle, which any thread may query. Its layout
//! crosses back as four `int[]`s of positions in the text Kotlin already holds, so no string is
//! copied back (see [`layout`]). A query answers with positions in a skin tone's list.

use std::ptr;

use emoji::data::{EmojiData, Span};
use emoji::suggest;
use emoji::tones::SKIN_TONES;
use jni_sys::{JNIEnv, jclass, jint, jintArray, jlong, jobjectArray, jstring};

use crate::handle::{free, into_handle, object};
use crate::jni::{self, Env, Result, illegal};

/// Where [`tones`]' hair style sits in its answer: the skin tone is the low byte.
const HAIR_SHIFT: u32 = 8;

fn int(value: usize) -> Result<i32> {
    i32::try_from(value).map_err(|_| illegal("too many emoji"))
}

fn skin_tone(tone: jint) -> Result<usize> {
    usize::try_from(tone)
        .ok()
        .filter(|&tone| tone < SKIN_TONES.len())
        .ok_or_else(|| illegal(format!("no skin tone {tone}")))
}

fn max_count(max: jint) -> Result<usize> {
    // Java's Stream.limit throws so for a negative count.
    usize::try_from(max).map_err(|_| illegal(max.to_string()))
}

/// Parses an emoji file's `text`, with the category ids (`EmojiCategory`'s, in order) in `ids`
/// cut by `id_lengths`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_EmojiNative_parse(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
    ids: jstring,
    id_lengths: jintArray,
) -> jlong {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            let text = env.read_string(text, "text")?;
            let ids = env.read_string(ids, "ids")?;
            let id_lengths = env.read::<i32>(id_lengths, "idLengths")?;
            let ids = jni::cut(&ids, &id_lengths)?;
            Ok(into_handle(EmojiData::parse(text, &ids)))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_EmojiNative_free(
    _: *mut JNIEnv,
    _: jclass,
    data: jlong,
) {
    // SAFETY: the Kotlin side frees each handle once, when nothing can reach it.
    unsafe { free::<EmojiData>(data) }
}

fn push_span(out: &mut Vec<i32>, span: Span) -> Result<()> {
    out.push(int(span.start)?);
    out.push(int(span.end)?);
    Ok(())
}

/// The parsed file as four `int[]`s:
/// - per emoji, seven: its value's start and end in the text, its name's, its skin tone and hair
///   style (ordinals) and how many keywords it has;
/// - per keyword, in order, its start and end;
/// - per set, its category (ordinal), its first emoji and how many it has;
/// - how many skin tones there are, how many emoji each shows, then those emoji, tone after tone.
fn layout(env: &Env, data: &EmojiData) -> Result<jobjectArray> {
    let (mut emojis, mut keywords) = (Vec::new(), Vec::new());
    for entry in &data.entries {
        push_span(&mut emojis, entry.value)?;
        push_span(&mut emojis, entry.name)?;
        emojis.push(int(entry.skin_tone)?);
        emojis.push(int(entry.hair_style)?);
        emojis.push(int(entry.keywords.len())?);
        for &keyword in &entry.keywords {
            push_span(&mut keywords, keyword)?;
        }
    }
    let mut sets = Vec::new();
    for set in &data.sets {
        sets.extend([int(set.category)?, int(set.start)?, int(set.len)?]);
    }
    let mut tones = vec![int(data.by_skin_tone.len())?];
    for list in &data.by_skin_tone {
        tones.push(int(list.len())?);
    }
    for list in &data.by_skin_tone {
        for &entry in list {
            tones.push(int(entry)?);
        }
    }
    env.make_objects(&[
        &|| env.make(&emojis),
        &|| env.make(&keywords),
        &|| env.make(&sets),
        &|| env.make(&tones),
    ])
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_EmojiNative_layout(
    env: *mut JNIEnv,
    _: jclass,
    data: jlong,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment for this call, and Kotlin a live handle (see
    // `handle::object`).
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            layout(env, object::<EmojiData>(data, "emoji data")?)
        })
    }
}

/// The query in the composing text ([`suggest::validate_query`]), or null if it holds none.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_EmojiNative_validateQuery(
    env: *mut JNIEnv,
    _: jclass,
    composing: jstring,
    prefix: jstring,
    min_length: jint,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let composing = env.read_string(composing, "composing")?;
            let prefix = env.read_string(prefix, "prefix")?;
            env.make_nullable_string(suggest::validate_query(&composing, &prefix, min_length))
        })
    }
}

/// The emoji for `query` among those shown for `tone`, best first ([`suggest::suggest`]), as
/// positions in that tone's list.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_EmojiNative_suggest(
    env: *mut JNIEnv,
    _: jclass,
    data: jlong,
    query: jstring,
    tone: jint,
    max: jint,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call, and Kotlin a live handle.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let data = object::<EmojiData>(data, "emoji data")?;
            let (tone, max) = (skin_tone(tone)?, max_count(max)?);
            let query = env.read_string(query, "query")?;
            let found = suggest::suggest(data, tone, &query, max);
            let found: Vec<i32> = found.iter().map(|&p| int(p)).collect::<Result<_>>()?;
            env.make(&found)
        })
    }
}

/// The skin tone and hair style of an emoji ([`emoji::tones::tones`]): the skin tone's ordinal,
/// with the hair style's shifted above it by [`HAIR_SHIFT`].
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_EmojiNative_tones(
    env: *mut JNIEnv,
    _: jclass,
    value: jstring,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            let (skin, hair) = emoji::tones::tones(&env.read_string(value, "value")?);
            Ok(int(skin)? | int(hair)? << HAIR_SHIFT)
        })
    }
}

/// Which of the emoji files, named in `names` cut by `lengths`, a locale reads
/// ([`emoji::asset::resolve`]): its position, or -1 for none.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_EmojiNative_resolveAsset(
    env: *mut JNIEnv,
    _: jclass,
    names: jstring,
    lengths: jintArray,
    locale: jobjectArray,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, -1, |env| {
            let names = env.read_string(names, "names")?;
            let lengths = env.read::<i32>(lengths, "lengths")?;
            let names = jni::cut(&names, &lengths)?;
            let locale = env.read_strings(locale, "locale")?;
            let [Some(language), Some(country), Some(variant)] = locale.as_slice() else {
                return Err(illegal("a locale is a language, a country and a variant"));
            };
            emoji::asset::resolve(&names, language, country, variant).map_or(Ok(-1), int)
        })
    }
}
