//! GIPHY's API, apart from the call itself: the URLs to ask it, and its answer read into GIFs
//! (<https://developers.giphy.com/docs/api/endpoint>). The answer is read as the keyboard's Kotlin
//! read it with kotlinx.serialization ([`json`](crate::json)), and its sizes as Kotlin's
//! `toFloatOrNull()` read them ([`java_float`](crate::java_float)).

use chars::is_blank;

use crate::java_float::to_float_or_null;
use crate::json::{Lexer, Result};
use crate::utf16;

/// Where GIPHY's GIF API is.
pub const BASE_URL: &str = "https://api.giphy.com/v1/gifs";
/// GIFs asked for at a time.
pub const LIMIT: u32 = 30;
/// The most a GIF may be rated.
pub const RATING: &str = "pg-13";
/// The longest search sent, in UTF-16 units.
pub const MAX_QUERY_LENGTH: usize = 50;

/// One GIF: a small animated preview for the panel, and the GIF that gets sent.
#[derive(Clone, Debug, PartialEq)]
pub struct Gif {
    pub id: Vec<u16>,
    pub title: Vec<u16>,
    pub preview_url: Vec<u16>,
    /// Width over height of the preview.
    pub aspect_ratio: f32,
    pub send_url: Vec<u16>,
    /// The GIF's page on giphy.com, typed instead when an app can't take GIFs.
    pub page_url: Vec<u16>,
}

/// One size of a GIF. GIPHY sends sizes as strings.
#[derive(Default)]
struct Rendition {
    url: Option<Vec<u16>>,
    webp: Option<Vec<u16>>,
    width: Option<Vec<u16>>,
    height: Option<Vec<u16>>,
}

#[derive(Default)]
struct Images {
    fixed_width: Option<Rendition>,
    downsized: Option<Rendition>,
}

#[derive(Default)]
struct Item {
    id: Vec<u16>,
    title: Vec<u16>,
    url: Vec<u16>,
    images: Images,
}

fn is(key: &[u16], name: &str) -> bool {
    key.iter().copied().eq(name.encode_utf16())
}

fn rendition(lexer: &mut Lexer) -> Result<Option<Rendition>> {
    if lexer.try_null() {
        return Ok(None);
    }
    let mut rendition = Rendition::default();
    lexer.object(|lexer, key| {
        let field = match () {
            () if is(key, "url") => &mut rendition.url,
            () if is(key, "webp") => &mut rendition.webp,
            () if is(key, "width") => &mut rendition.width,
            () if is(key, "height") => &mut rendition.height,
            () => return Ok(false),
        };
        *field = lexer.nullable_string()?;
        Ok(true)
    })?;
    Ok(Some(rendition))
}

fn images(lexer: &mut Lexer) -> Result<Images> {
    let mut images = Images::default();
    lexer.object(|lexer, key| {
        let field = match () {
            () if is(key, "fixed_width") => &mut images.fixed_width,
            () if is(key, "downsized") => &mut images.downsized,
            () => return Ok(false),
        };
        *field = rendition(lexer)?;
        Ok(true)
    })?;
    Ok(images)
}

fn item(lexer: &mut Lexer) -> Result<Item> {
    let mut item = Item::default();
    lexer.object(|lexer, key| {
        match () {
            () if is(key, "id") => item.id = lexer.string()?,
            () if is(key, "title") => item.title = lexer.string()?,
            () if is(key, "url") => item.url = lexer.string()?,
            () if is(key, "images") => item.images = images(lexer)?,
            () => return Ok(false),
        }
        Ok(true)
    })?;
    Ok(item)
}

/// The items of a search or trending answer, or why it is unreadable.
fn items(body: &[u16]) -> Result<Vec<Item>> {
    let mut lexer = Lexer::new(body);
    let mut data = Vec::new();
    lexer.object(|lexer, key| {
        if !is(key, "data") {
            return Ok(false);
        }
        // A repeated "data" replaces the one before.
        data.clear();
        lexer.list(|lexer| {
            data.push(item(lexer)?);
            Ok(())
        })?;
        Ok(true)
    })?;
    lexer.expect_eof()?;
    Ok(data)
}

/// The GIF an item shows, if it has an id and the renditions the panel needs.
fn gif(item: Item) -> Option<Gif> {
    if is_blank(&item.id) {
        return None;
    }
    let Images {
        fixed_width,
        downsized,
    } = item.images;
    let preview = fixed_width?;
    let preview_url = preview.webp.or_else(|| preview.url.clone())?;
    let width = to_float_or_null(preview.width.as_deref()?)?;
    let height = to_float_or_null(preview.height.as_deref()?).filter(|&it| it > 0.0)?;
    let send_url = downsized.and_then(|it| it.url).or(preview.url)?;
    Some(Gif {
        id: item.id,
        title: item.title,
        preview_url,
        aspect_ratio: width / height,
        send_url,
        page_url: item.url,
    })
}

/// The GIFs in a search or trending answer, skipping any without an id or the renditions the
/// panel needs, and repeats of an id. `Err` if the body isn't the JSON expected.
pub fn parse(body: &[u16]) -> Result<Vec<Gif>> {
    let mut gifs: Vec<Gif> = Vec::new();
    for gif in items(body)?.into_iter().filter_map(gif) {
        if !gifs.iter().any(|it| it.id == gif.id) {
            gifs.push(gif);
        }
    }
    Ok(gifs)
}

/// `value` as Java's `URLEncoder.encode(value, "UTF-8")` writes it: letters, digits and `.-*_`
/// as they are, a space as `+`, and every other byte of its UTF-8 as `%XX`, where a surrogate
/// without its pair is `?`.
pub fn encode(value: &[u16]) -> Vec<u16> {
    let mut encoded = Vec::with_capacity(value.len());
    let mut bytes = [0; 4];
    for unit in char::decode_utf16(value.iter().copied()) {
        let unit = unit.unwrap_or('?');
        match unit {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '*' | '_' => encoded.push(unit as u16),
            ' ' => encoded.push(u16::from(b'+')),
            _ => {
                for byte in unit.encode_utf8(&mut bytes).bytes() {
                    encoded.extend(format!("%{byte:02X}").encode_utf16());
                }
            }
        }
    }
    encoded
}

/// The URL that asks `endpoint` with these `params`, then the key and GIPHY's settings, each a
/// name and its value. A setting named in `params` keeps its place there and takes the
/// setting's value, as Kotlin's `Map.plus` does.
pub fn url(api_key: &[u16], endpoint: &[u16], params: &[(Vec<u16>, Vec<u16>)]) -> Vec<u16> {
    let mut all = params.to_vec();
    let settings = [
        ("api_key", api_key.to_vec()),
        ("limit", utf16(&LIMIT.to_string())),
        ("rating", utf16(RATING)),
        ("lang", utf16("en")),
    ];
    for (name, value) in settings {
        match all.iter_mut().find(|(it, _)| is(it, name)) {
            Some(param) => param.1 = value,
            None => all.push((utf16(name), value)),
        }
    }
    let mut url = utf16(BASE_URL);
    url.push(u16::from(b'/'));
    url.extend_from_slice(endpoint);
    url.push(u16::from(b'?'));
    for (i, (name, value)) in all.iter().enumerate() {
        if i > 0 {
            url.push(u16::from(b'&'));
        }
        url.extend_from_slice(name);
        url.push(u16::from(b'='));
        url.extend(encode(value));
    }
    url
}

/// The URL that asks for trending GIFs.
pub fn trending_url(api_key: &[u16]) -> Vec<u16> {
    url(api_key, &utf16("trending"), &[])
}

/// The URL that searches for `query`, cut to [`MAX_QUERY_LENGTH`] units.
pub fn search_url(api_key: &[u16], query: &[u16]) -> Vec<u16> {
    let query = &query[..query.len().min(MAX_QUERY_LENGTH)];
    url(api_key, &utf16("search"), &[(utf16("q"), query.to_vec())])
}

#[cfg(test)]
mod tests;
