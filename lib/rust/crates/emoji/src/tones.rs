//! An emoji's skin tone and hair style: `Emoji`'s `skinTone` and `hairStyle`, the first of
//! `EmojiSkinTone`'s and `EmojiHairStyle`'s entries whose code point the emoji holds.

use chars::text::code_points;

/// `EmojiSkinTone`'s ids, in its order: `DEFAULT` (0), then the five Fitzpatrick modifiers.
pub const SKIN_TONES: [u32; 6] = [0x0, 0x1F3FB, 0x1F3FC, 0x1F3FD, 0x1F3FE, 0x1F3FF];

/// `EmojiHairStyle`'s ids, in its order: `DEFAULT` (0), red, curly, white hair and bald.
pub const HAIR_STYLES: [u32; 5] = [0x0, 0x1F9B0, 0x1F9B1, 0x1F9B2, 0x1F9B3];

/// The skin tone and hair style of the emoji `value`, as ordinals of `EmojiSkinTone` and
/// `EmojiHairStyle`: the first entry whose id is one of `value`'s code points (Java's
/// `codePoints()`, so a lone surrogate is one), else `DEFAULT`. `DEFAULT`'s id is 0, so a value
/// holding U+0000 is `DEFAULT` whatever else it holds, as in Kotlin.
pub fn tones(value: &[u16]) -> (usize, usize) {
    let code_points: Vec<u32> = code_points(value).map(|(_, cp)| cp).collect();
    let first = |ids: &[u32]| {
        ids.iter()
            .position(|id| code_points.contains(id))
            .unwrap_or(0)
    };
    (first(&SKIN_TONES), first(&HAIR_STYLES))
}

#[cfg(test)]
mod tests;
