//! Which emoji file a locale reads: `EmojiData.resolveEmojiAssetPath`'s choice among the files in
//! `assets/ime/media/emoji/`.

/// The position in `assets` of the file for this locale: `{language}_{country}_{variant}.txt` if
/// both country and variant are given (not blank), else or failing that `{language}_{country}.txt`
/// if the country is, then `{language}.txt`, with the language in lower case (`lowercase()`), and
/// `None` if there is no such file.
pub fn resolve(
    assets: &[&[u16]],
    language: &[u16],
    country: &[u16],
    variant: &[u16],
) -> Option<usize> {
    let language = chars::lowercase(language);
    let country = (!chars::is_blank(country)).then_some(country);
    let variant = (!chars::is_blank(variant)).then_some(variant);
    let find = |parts: &[&[u16]]| {
        let mut name = parts.join(&(b'_' as u16));
        name.extend(".txt".encode_utf16());
        assets.iter().position(|&asset| asset == name.as_slice())
    };
    if let (Some(country), Some(variant)) = (country, variant)
        && let Some(found) = find(&[&language, country, variant])
    {
        return Some(found);
    }
    if let Some(country) = country
        && let Some(found) = find(&[&language, country])
    {
        return Some(found);
    }
    find(&[&language])
}

#[cfg(test)]
mod tests;
