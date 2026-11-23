//! A theme's own fonts: the files its `@font` rules name, read once, checked, and kept in a
//! process-wide registry, so every [`Text`](crate::text::Text) (the session's, each raster's)
//! draws one under the same [`Font::Theme`] id and caches keyed by it never go stale.
//!
//! Kotlin resolves each rule's `src` URI through the theme's asset resolver and passes the
//! absolute paths; nothing here resolves a URI itself. A file that is missing, too big or not a
//! whole font leaves its rule without that face, and a rule with no face left is no font at all:
//! elements naming it draw in the default chain, as Compose draws them without a `FontFamily`.
use crate::text::{Font, FontBlob};
use read_fonts::{FileRef, TableProvider};
use snygg::{
    resolve::{Properties, get},
    text,
    value::{Encoder, Value},
};
use std::sync::{Arc, Mutex};

/// One face of a theme font: its file and the weight and style its `@font` set gives it.
#[derive(Clone, Debug)]
pub struct Face {
    pub blob: FontBlob,
    /// `font-weight`, 400 when the set names none, as Compose's `Font` defaults it.
    pub weight: i32,
    /// `font-style: italic`.
    pub italic: bool,
}

impl PartialEq for Face {
    fn eq(&self, other: &Self) -> bool {
        (self.weight, self.italic) == (other.weight, other.italic)
            && self.blob.data() == other.blob.data()
    }
}

/// Font files bigger than this are not read: a theme's font is a few hundred kilobytes, and a
/// CJK font a few dozen megabytes no keyboard theme should carry.
pub const MAX_FILE: u64 = 32 << 20;
/// The distinct families one process keeps. Each is kept for good, since a `Text` registered it
/// under an id its caches use; past this a new one draws in the default chain.
pub const MAX_FAMILIES: usize = 64;

static FAMILIES: Mutex<Vec<Arc<[Face]>>> = Mutex::new(Vec::new());

fn families() -> std::sync::MutexGuard<'static, Vec<Arc<[Face]>>> {
    // Nothing panics while holding it, but a poisoned registry is still a whole one.
    FAMILIES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The faces registered as theme font `id`.
pub(crate) fn family(id: u32) -> Option<Arc<[Face]>> {
    families().get(id as usize).cloned()
}

/// `faces` as one theme font: the id the same faces already have, or a new one.
fn intern(faces: Vec<Face>) -> Option<Font> {
    let mut all = families();
    if let Some(id) = all.iter().position(|f| **f == *faces) {
        return Some(Font::Theme(id as u32));
    }
    if all.len() >= MAX_FAMILIES {
        return None;
    }
    all.push(faces.into());
    Some(Font::Theme(all.len() as u32 - 1))
}

/// The theme's fonts by name: each `@font` rule in `rules` with at least one face whose `src`
/// resolves through `files` (URI, absolute path) to a readable whole font. Rules of one name
/// share a family, as Kotlin's `compileFrom` merges them.
pub fn load(
    rules: &[(snygg::Text, Vec<Properties>)],
    files: &[(String, String)],
) -> Vec<(snygg::Text, Font)> {
    let mut named: Vec<(snygg::Text, Vec<Face>)> = Vec::new();
    for (name, sets) in rules {
        let faces = sets.iter().filter_map(|set| face(set, files));
        match named.iter_mut().find(|(n, _)| n == name) {
            Some((_, known)) => known.extend(faces),
            None => named.push((name.clone(), faces.collect())),
        }
    }
    named
        .into_iter()
        .filter(|(_, faces)| !faces.is_empty())
        .filter_map(|(name, faces)| Some((name, intern(faces)?)))
        .collect()
}

fn face(set: &Properties, files: &[(String, String)]) -> Option<Face> {
    let Some(Value::Uri(uri)) = get(set, &text("src")) else {
        return None;
    };
    let uri = String::from_utf16(uri).ok()?;
    let (_, path) = files.iter().find(|(u, _)| *u == uri)?;
    Some(Face {
        blob: read(path)?,
        weight: match get(set, &text("font-weight")) {
            Some(Value::FontWeight(w)) => *w,
            _ => 400,
        },
        italic: matches!(get(set, &text("font-style")),
            Some(Value::Enum(Encoder::FontStyle, s)) if *s == text("italic")),
    })
}

/// The font file at `path`, unless it is missing, bigger than [`MAX_FILE`] or not [`whole`].
pub fn read(path: &str) -> Option<FontBlob> {
    let size = std::fs::metadata(path).ok()?.len();
    if size > MAX_FILE {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    whole(&bytes).then(|| FontBlob::from(bytes))
}

/// Whether `bytes` is a whole font or collection: every face's tables inside the file and its
/// `head`, `hhea`, `hmtx`, `maxp` and `cmap` readable. A truncated file fails the first, garbage
/// fails to parse at all. Outlines may be `glyf`, CFF or only colour bitmaps.
pub fn whole(bytes: &[u8]) -> bool {
    let Ok(file) = FileRef::new(bytes) else {
        return false;
    };
    let mut fonts = file.fonts().peekable();
    fonts.peek().is_some()
        && fonts.all(|font| {
            let Ok(font) = font else { return false };
            let inside = font.table_directory.table_records().iter().all(|r| {
                (r.offset() as usize)
                    .checked_add(r.length() as usize)
                    .is_some_and(|end| end <= bytes.len())
            });
            inside
                && font.head().is_ok()
                && font.hhea().is_ok()
                && font.hmtx().is_ok()
                && font.maxp().is_ok()
                && font.cmap().is_ok()
        })
}
