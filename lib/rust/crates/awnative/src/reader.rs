//! JNI entry points for reading a page aloud (the `reader` crate), called by `ReaderNative` in
//! lib/rust's Kotlin. Blocks' edges cross as one `int[]`, each block's top then its bottom, and an
//! answer that may be none is -1 or null.

use std::num::NonZeroUsize;
use std::ptr;
use std::sync::Mutex;

use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jbooleanArray, jclass, jint, jintArray, jlong};
use jni_sys::{jobjectArray, jstring};
use reader::follow_along;
use reader::page_text::{self, Edges, Piece};
use reader::speaker_key::{self, Action, SpeakerKey};
use reader::voices::{self, Facts};

use crate::handle::{free, into_handle, with_locked};
use crate::jni::{self, Env, Result, boolean, cut, illegal};

/// No block, no action.
const NONE: jint = -1;

/// An index into a Java array or string, or a count of one, which fits in an `int`.
fn index(value: usize) -> jint {
    value as jint
}

/// The edges in `edges`, each block's top then its bottom.
fn read_edges(env: &Env, edges: jintArray) -> Result<Vec<Edges>> {
    let values = env.read::<i32>(edges, "edges")?;
    let (pairs, rest) = values.as_chunks::<2>();
    if !rest.is_empty() {
        return Err(illegal("a top without its bottom"));
    }
    Ok((pairs.iter())
        .map(|&[top, bottom]| Edges { top, bottom })
        .collect())
}

fn visible(top: jint, bottom: jint) -> Edges {
    Edges { top, bottom }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_maxUtterance(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    index(page_text::MAX_UTTERANCE)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_recent(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    index(page_text::RECENT)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_holdMs(
    _: *mut JNIEnv,
    _: jclass,
) -> jlong {
    speaker_key::HOLD_MS
}

/// The first block whose `edges` show between `visibleTop` and `visibleBottom`
/// (`page_text::first_on_screen`), or -1.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_firstOnScreen(
    env: *mut JNIEnv,
    _: jclass,
    edges: jintArray,
    visible_top: jint,
    visible_bottom: jint,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, NONE, |env| {
            let blocks = read_edges(env, edges)?;
            let first = page_text::first_on_screen(&blocks, visible(visible_top, visible_bottom));
            Ok(first.map_or(NONE, index))
        })
    }
}

/// Whether every block whose `edges` these are shows (`page_text::all_shown`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_allShown(
    env: *mut JNIEnv,
    _: jclass,
    edges: jintArray,
    visible_top: jint,
    visible_bottom: jint,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, JNI_FALSE, |env| {
            let blocks = read_edges(env, edges)?;
            let shown = page_text::all_shown(&blocks, visible(visible_top, visible_bottom));
            Ok(boolean(shown))
        })
    }
}

/// The pieces of text with these `texts`, `edges` and `scrolls` made into blocks to read
/// (`page_text::blocks`): their texts, with each one's first piece, top and bottom written to
/// `places`: count, then first piece/top/bottom/UTF-16 length per block. Text returns as one string.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_join(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
    lengths: jintArray,
    edges: jintArray,
    scrolls: jbooleanArray,
    places: jintArray,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let text = env.read_string(text, "texts")?;
            let lengths = env.read::<i32>(lengths, "lengths")?;
            let texts = cut(&text, &lengths)?;
            let edges = read_edges(env, edges)?;
            let scrolls = env.read::<jboolean>(scrolls, "scrolls")?;
            if edges.len() != texts.len() || scrolls.len() != texts.len() {
                return Err(illegal("not as many edges and scrolls as texts"));
            }
            let pieces: Vec<Piece> = (texts.iter().zip(edges).zip(scrolls))
                .map(|((text, edges), scrolls)| Piece {
                    text,
                    edges,
                    scrolls: scrolls != 0,
                })
                .collect();
            let blocks = page_text::blocks(&pieces);
            let mut values = vec![index(blocks.len())];
            values.extend(blocks.iter().flat_map(|it| {
                [
                    index(it.first),
                    it.edges.top,
                    it.edges.bottom,
                    index(it.text.len()),
                ]
            }));
            env.write(places, &values, "places")?;
            env.make_string(
                &blocks
                    .iter()
                    .flat_map(|it| it.text.iter().copied())
                    .collect::<Vec<_>>(),
            )
        })
    }
}

/// Where `selected` starts among the blocks with these `texts` and `edges` (`page_text::find`):
/// its block and its offset in that block's text, or null if it isn't there.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_find(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
    lengths: jintArray,
    edges: jintArray,
    selected: jstring,
    visible_top: jint,
    visible_bottom: jint,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let text = env.read_string(text, "texts")?;
            let lengths = env.read::<i32>(lengths, "lengths")?;
            let texts = cut(&text, &lengths)?;
            let edges = read_edges(env, edges)?;
            if edges.len() != texts.len() {
                return Err(illegal("not as many edges as texts"));
            }
            let blocks: Vec<(&[u16], Edges)> = texts.iter().copied().zip(edges).collect();
            let selected = env.read_string(selected, "selected")?;
            let visible = visible(visible_top, visible_bottom);
            match page_text::find(&blocks, &selected, visible) {
                Some(start) => env.make(&[index(start.block), index(start.offset)]),
                None => Ok(ptr::null_mut()),
            }
        })
    }
}

/// Where to carry on among the blocks with these `texts` after a scroll, given the texts `read`
/// so far (`page_text::resume_after`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_resumeAfter(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
    lengths: jintArray,
    text_count: jint,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            let text = env.read_string(text, "texts and read")?;
            let lengths = env.read::<i32>(lengths, "lengths")?;
            let fields = cut(&text, &lengths)?;
            let count = usize::try_from(text_count).map_err(|_| illegal("negative text count"))?;
            let (texts, read) = fields
                .split_at_checked(count)
                .ok_or_else(|| illegal("missing texts"))?;
            Ok(index(page_text::resume_after(texts, read)))
        })
    }
}

/// `text`, whose sentences are `sentenceLengths` long, cut into pieces of at most `max` units for
/// the speech engine (`page_text::utterances`). `max` must be positive.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_utterances(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
    sentence_lengths: jintArray,
    max: jint,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let max = (usize::try_from(max).ok().and_then(NonZeroUsize::new))
                .ok_or_else(|| illegal(format!("max must be positive, was {max}")))?;
            let text = env.read_string(text, "text")?;
            let lengths = env.read::<i32>(sentence_lengths, "sentenceLengths")?;
            let sentences = cut(&text, &lengths)?;
            env.make_strings(&page_text::utterances(&sentences, max))
        })
    }
}

/// Whether a piece from `first` to `last` fits in a view from `top` to `bottom`
/// (`follow_along::fits`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_fits(
    _: *mut JNIEnv,
    _: jclass,
    first: jint,
    last: jint,
    top: jint,
    bottom: jint,
) -> jboolean {
    boolean(follow_along::fits(first, last, top, bottom))
}

/// How many numbers `offlineVoice` takes for each voice.
const VOICE_NUMBERS: usize = 4;

/// Which of the engine's voices may read (`voices::offline_voice`), or -1 for none. `locales` is
/// each voice's language then its country; `numbers` is each voice's needs-network and installed
/// (1 or 0), quality and latency; `preferred` is the default voice's index, or -1.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_offlineVoice(
    env: *mut JNIEnv,
    _: jclass,
    locales: jobjectArray,
    numbers: jintArray,
    language: jstring,
    country: jstring,
    preferred: jint,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, NONE, |env| {
            let locales = env.read_strings(locales, "locales")?;
            let numbers = env.read::<i32>(numbers, "numbers")?;
            let (rows, rest) = numbers.as_chunks::<VOICE_NUMBERS>();
            if !rest.is_empty() || locales.len() != rows.len() * 2 {
                return Err(illegal("two locale parts and four numbers for each voice"));
            }
            let locales: Vec<Vec<u16>> =
                locales.into_iter().map(Option::unwrap_or_default).collect();
            let voices: Vec<Facts> = (locales.as_chunks::<2>().0.iter().zip(rows))
                .map(
                    |([language, country], &[network, installed, quality, latency])| Facts {
                        language,
                        country,
                        needs_network: network != 0,
                        installed: installed != 0,
                        quality,
                        latency,
                    },
                )
                .collect();
            let language = env.read_string(language, "language")?;
            let country = env.read_string(country, "country")?;
            let preferred = usize::try_from(preferred).ok();
            Ok(voices::offline_voice(&voices, &language, &country, preferred).map_or(NONE, index))
        })
    }
}

/// A new speaker key, not pressed.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_newSpeakerKey(
    _: *mut JNIEnv,
    _: jclass,
) -> jlong {
    into_handle(Mutex::new(SpeakerKey::default()))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_freeSpeakerKey(
    _: *mut JNIEnv,
    _: jclass,
    key: jlong,
) {
    // SAFETY: the Kotlin side frees each key once, when nothing can reach it.
    unsafe { free::<Mutex<SpeakerKey>>(key) }
}

/// An action's number, as `SpeakerKeyState.Action` orders them, or -1 for none.
fn action(action: Option<Action>) -> jint {
    match action {
        None => NONE,
        Some(Action::Stop) => 0,
        Some(Action::FromScreen) => 1,
        Some(Action::FromTop) => 2,
    }
}

/// The key behind `key` was pressed at `now` (`SpeakerKey::down`): what that asks for.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_speakerKeyDown(
    env: *mut JNIEnv,
    _: jclass,
    key: jlong,
    reading: jboolean,
    now: jlong,
) -> jint {
    // SAFETY: the JVM passes its environment; Kotlin holds the key while it calls.
    unsafe {
        with_locked(env, key, "speaker key", NONE, |key: &mut SpeakerKey, _| {
            Ok(action(key.down(reading != 0, now)))
        })
    }
}

/// The key behind `key` was lifted at `now` (`SpeakerKey::up`): what that asks for.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_speakerKeyUp(
    env: *mut JNIEnv,
    _: jclass,
    key: jlong,
    now: jlong,
) -> jint {
    // SAFETY: the JVM passes its environment; Kotlin holds the key while it calls.
    unsafe {
        with_locked(env, key, "speaker key", NONE, |key: &mut SpeakerKey, _| {
            Ok(action(key.up(now)))
        })
    }
}

/// The finger slid off the key behind `key` (`SpeakerKey::cancel`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderNative_speakerKeyCancel(
    env: *mut JNIEnv,
    _: jclass,
    key: jlong,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the key while it calls.
    unsafe {
        with_locked(env, key, "speaker key", (), |key: &mut SpeakerKey, _| {
            key.cancel();
            Ok(())
        })
    }
}
