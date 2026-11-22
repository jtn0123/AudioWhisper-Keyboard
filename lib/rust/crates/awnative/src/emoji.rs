//! Emoji data and suggestions (the `emoji` crate) for `EmojiNative`, whose generated entry points
//! call these or the crate directly.
//!
//! An emoji file is parsed once into a read-only handle, which any thread may query. Its entries
//! cross back as one row of strings each, its sets as three ints each, and each skin tone's emoji
//! as a row of positions. A query answers with positions in a skin tone's list.

use emoji::data::{EmojiData, Span};
use emoji::suggest;
use emoji::tones::SKIN_TONES;

use crate::handle::{into_handle, object};
use crate::jni::{Result, decimal, illegal};

fn int(value: usize) -> Result<i32> {
    i32::try_from(value).map_err(|_| illegal("too many emoji"))
}

fn skin_tone(tone: i32) -> Result<usize> {
    usize::try_from(tone)
        .ok()
        .filter(|&tone| tone < SKIN_TONES.len())
        .ok_or_else(|| illegal(format!("no skin tone {tone}")))
}

fn max_count(max: i32) -> Result<usize> {
    // Java's Stream.limit throws so for a negative count.
    usize::try_from(max).map_err(|_| illegal(max.to_string()))
}

/// The emoji data behind `data`.
fn data<'a>(data: i64) -> Result<&'a EmojiData> {
    // SAFETY: Kotlin holds the table while it calls, so it hasn't been freed.
    unsafe { object::<EmojiData>(data, "emoji data") }
}

/// Parses an emoji file's `text`, with `category_ids` the category ids (`EmojiCategory`'s, in order).
pub fn parse(text: &[u16], category_ids: &[Vec<u16>]) -> i64 {
    let ids: Vec<&[u16]> = category_ids.iter().map(Vec::as_slice).collect();
    into_handle(EmojiData::parse(text.to_vec(), &ids))
}

pub fn free(data: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each handle once, when nothing can reach it.
    unsafe { crate::handle::free::<EmojiData>(data) };
    Ok(())
}

/// Each emoji as a row: its value, its name, its skin tone and hair style (ordinals), then its
/// keywords.
pub fn entries(data: i64) -> Result<Vec<Vec<Vec<u16>>>> {
    let data = self::data(data)?;
    let cut = |span: Span| data.text[span.start..span.end].to_vec();
    let rows = data.entries.iter().map(|entry| {
        let head = [
            cut(entry.value),
            cut(entry.name),
            decimal(entry.skin_tone),
            decimal(entry.hair_style),
        ];
        head.into_iter()
            .chain(entry.keywords.iter().map(|&it| cut(it)))
            .collect()
    });
    Ok(rows.collect())
}

/// Each set as three ints: its category (ordinal), its first emoji and how many it has.
pub fn sets(data: i64) -> Result<Vec<i32>> {
    let mut sets = Vec::new();
    for set in &self::data(data)?.sets {
        sets.extend([int(set.category)?, int(set.start)?, int(set.len)?]);
    }
    Ok(sets)
}

/// For each skin tone, by ordinal, the positions of the emoji shown with it chosen.
pub fn by_skin_tone(data: i64) -> Result<Vec<Vec<i32>>> {
    let lists = self::data(data)?.by_skin_tone.iter();
    lists
        .map(|list| list.iter().map(|&it| int(it)).collect())
        .collect()
}

/// The query in the composing text ([`suggest::validate_query`]), or null if it holds none.
pub fn validate_query(composing: &[u16], prefix: &[u16], min_length: i32) -> Option<Vec<u16>> {
    suggest::validate_query(composing, prefix, min_length).map(<[u16]>::to_vec)
}

/// The emoji for `query` among those shown for `tone`, best first ([`suggest::suggest`]), as
/// positions in that tone's list.
pub fn suggest(data: i64, query: &[u16], tone: i32, max: i32) -> Result<Vec<i32>> {
    let data = self::data(data)?;
    let found = suggest::suggest(data, skin_tone(tone)?, query, max_count(max)?);
    found.into_iter().map(int).collect()
}

/// The skin tone and hair style of an emoji ([`emoji::tones::tones`]), as ordinals.
pub fn tones(value: &[u16]) -> Result<Vec<i32>> {
    let (skin, hair) = emoji::tones::tones(value);
    Ok(vec![int(skin)?, int(hair)?])
}

/// Which of the emoji files `assets` a locale reads ([`emoji::asset::resolve`]): its position,
/// or -1 for none.
pub fn resolve_asset(
    assets: &[Vec<u16>],
    language: &[u16],
    country: &[u16],
    variant: &[u16],
) -> Result<i32> {
    let assets: Vec<&[u16]> = assets.iter().map(Vec::as_slice).collect();
    emoji::asset::resolve(&assets, language, country, variant).map_or(Ok(-1), int)
}
