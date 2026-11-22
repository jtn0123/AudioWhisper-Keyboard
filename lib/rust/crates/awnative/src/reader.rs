//! Reading a page aloud (the `reader` crate), for `ReaderNative`, whose generated entry points
//! call these. Blocks' edges cross as one `int[]`, each block's top then its bottom, and an answer
//! that may be none is -1 or null.

use std::num::NonZeroUsize;
use std::sync::{Mutex, MutexGuard};

use reader::follow_along;
use reader::page_text::{self, Edges, Piece};
use reader::speaker_key::{self, Action, SpeakerKey};
use reader::voices::{self, Facts};

use crate::handle::{free, into_handle, locked};
use crate::jni::{Result, cut, decimal, illegal};

/// No block, no action.
const NONE: i32 = -1;

/// An index into a Java array or string, or a count of one, which fits in an `int`.
fn index(value: usize) -> i32 {
    value as i32
}

/// The edges in `edges`, each block's top then its bottom.
fn read_edges(edges: &[i32]) -> Result<Vec<Edges>> {
    let (pairs, rest) = edges.as_chunks::<2>();
    if !rest.is_empty() {
        return Err(illegal("a top without its bottom"));
    }
    Ok((pairs.iter())
        .map(|&[top, bottom]| Edges { top, bottom })
        .collect())
}

fn visible(top: i32, bottom: i32) -> Edges {
    Edges { top, bottom }
}

pub fn max_utterance() -> i32 {
    index(page_text::MAX_UTTERANCE)
}

pub fn recent() -> i32 {
    index(page_text::RECENT)
}

pub fn hold_ms() -> i64 {
    speaker_key::HOLD_MS
}

/// The first block whose `edges` show between `visible_top` and `visible_bottom`
/// (`page_text::first_on_screen`), or -1.
pub fn first_on_screen(edges: &[i32], visible_top: i32, visible_bottom: i32) -> Result<i32> {
    let blocks = read_edges(edges)?;
    let first = page_text::first_on_screen(&blocks, visible(visible_top, visible_bottom));
    Ok(first.map_or(NONE, index))
}

/// Whether every block whose `edges` these are shows (`page_text::all_shown`).
pub fn all_shown(edges: &[i32], visible_top: i32, visible_bottom: i32) -> Result<bool> {
    let blocks = read_edges(edges)?;
    Ok(page_text::all_shown(
        &blocks,
        visible(visible_top, visible_bottom),
    ))
}

/// The pieces of text with these `texts`, `edges` and `scrolls` made into blocks to read
/// (`page_text::blocks`): four strings a block, its first piece, its text, its top and its bottom.
pub fn join(texts: &[Vec<u16>], edges: &[i32], scrolls: &[u8]) -> Result<Vec<Vec<u16>>> {
    let edges = read_edges(edges)?;
    if edges.len() != texts.len() || scrolls.len() != texts.len() {
        return Err(illegal("not as many edges and scrolls as texts"));
    }
    let pieces: Vec<Piece> = (texts.iter().zip(edges).zip(scrolls))
        .map(|((text, edges), &scrolls)| Piece {
            text,
            edges,
            scrolls: scrolls != 0,
        })
        .collect();
    let blocks = page_text::blocks(&pieces);
    Ok(blocks
        .into_iter()
        .flat_map(|it| {
            [
                decimal(it.first),
                it.text,
                decimal(it.edges.top),
                decimal(it.edges.bottom),
            ]
        })
        .collect())
}

/// Where `selected` starts among the blocks with these `texts` and `edges` (`page_text::find`):
/// its block and its offset in that block's text, or null if it isn't there.
pub fn find(
    texts: &[Vec<u16>],
    edges: &[i32],
    selected: &[u16],
    visible_top: i32,
    visible_bottom: i32,
) -> Result<Option<Vec<i32>>> {
    let edges = read_edges(edges)?;
    if edges.len() != texts.len() {
        return Err(illegal("not as many edges as texts"));
    }
    let blocks: Vec<(&[u16], Edges)> = texts.iter().map(Vec::as_slice).zip(edges).collect();
    let start = page_text::find(&blocks, selected, visible(visible_top, visible_bottom));
    Ok(start.map(|start| vec![index(start.block), index(start.offset)]))
}

/// Where to carry on among the blocks with these `texts` after a scroll, given the texts `read`
/// so far (`page_text::resume_after`).
pub fn resume_after(texts: &[Vec<u16>], read: &[Vec<u16>]) -> i32 {
    let texts: Vec<&[u16]> = texts.iter().map(Vec::as_slice).collect();
    let read: Vec<&[u16]> = read.iter().map(Vec::as_slice).collect();
    index(page_text::resume_after(&texts, &read))
}

/// `text`, whose sentences are `sentence_lengths` long, cut into pieces of at most `max` units for
/// the speech engine (`page_text::utterances`). `max` must be positive.
pub fn utterances(text: &[u16], sentence_lengths: &[i32], max: i32) -> Result<Vec<Vec<u16>>> {
    let max = (usize::try_from(max).ok().and_then(NonZeroUsize::new))
        .ok_or_else(|| illegal(format!("max must be positive, was {max}")))?;
    let sentences = cut(text, sentence_lengths)?;
    Ok(page_text::utterances(&sentences, max))
}

/// Whether a piece from `first` to `last` fits in a view from `top` to `bottom`
/// (`follow_along::fits`).
pub fn fits(first: i32, last: i32, top: i32, bottom: i32) -> bool {
    follow_along::fits(first, last, top, bottom)
}

/// How many numbers `offlineVoice` takes for each voice.
const VOICE_NUMBERS: usize = 4;

/// Which of the engine's voices may read (`voices::offline_voice`), or -1 for none. `locales` is
/// each voice's language then its country; `numbers` is each voice's needs-network and installed
/// (1 or 0), quality and latency; `preferred` is the default voice's index, or -1.
pub fn offline_voice(
    locales: &[Vec<u16>],
    numbers: &[i32],
    language: &[u16],
    country: &[u16],
    preferred: i32,
) -> Result<i32> {
    let (rows, rest) = numbers.as_chunks::<VOICE_NUMBERS>();
    if !rest.is_empty() || locales.len() != rows.len() * 2 {
        return Err(illegal("two locale parts and four numbers for each voice"));
    }
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
    let preferred = usize::try_from(preferred).ok();
    Ok(voices::offline_voice(&voices, language, country, preferred).map_or(NONE, index))
}

/// A new speaker key, not pressed.
pub fn new_speaker_key() -> i64 {
    into_handle(Mutex::new(SpeakerKey::default()))
}

pub fn free_speaker_key(key: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each key once, when nothing can reach it.
    unsafe { free::<Mutex<SpeakerKey>>(key) };
    Ok(())
}

fn key_of<'a>(key: i64) -> Result<MutexGuard<'a, SpeakerKey>> {
    // SAFETY: Kotlin holds the key while it calls, so it hasn't been freed.
    unsafe { locked(key, "speaker key") }
}

/// An action's number, as `SpeakerKeyState.Action` orders them, or -1 for none.
fn action(action: Option<Action>) -> i32 {
    match action {
        None => NONE,
        Some(Action::Stop) => 0,
        Some(Action::FromScreen) => 1,
        Some(Action::FromTop) => 2,
    }
}

/// The key behind `key` was pressed at `now` (`SpeakerKey::down`): what that asks for.
pub fn speaker_key_down(key: i64, reading: bool, now: i64) -> Result<i32> {
    Ok(action(key_of(key)?.down(reading, now)))
}

/// The key behind `key` was lifted at `now` (`SpeakerKey::up`): what that asks for.
pub fn speaker_key_up(key: i64, now: i64) -> Result<i32> {
    Ok(action(key_of(key)?.up(now)))
}

/// The finger slid off the key behind `key` (`SpeakerKey::cancel`).
pub fn speaker_key_cancel(key: i64) -> Result<()> {
    key_of(key)?.cancel();
    Ok(())
}
