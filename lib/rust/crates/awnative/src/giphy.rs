//! GIPHY's URLs and answers (`gif::giphy`) for `GiphyNative`, whose generated entry points call
//! these or the crate directly.

use gif::giphy::{self, Gif};

use crate::jni::{Result, illegal};

/// The GIFs in GIPHY's answer `body`, six strings a GIF: id, title, preview URL, aspect ratio,
/// send URL and page URL. The ratio is written as Kotlin's `toFloat` reads it back exactly.
/// Throws IllegalArgumentException if the body isn't the JSON expected, as kotlinx.serialization
/// did.
pub fn parse(body: &[u16]) -> Result<Vec<Vec<u16>>> {
    let gifs = giphy::parse(body).map_err(|_| illegal("GIPHY's answer is unreadable"))?;
    Ok(gifs.into_iter().flat_map(row).collect())
}

fn row(gif: Gif) -> [Vec<u16>; 6] {
    let ratio = float(gif.aspect_ratio).encode_utf16().collect();
    [
        gif.id,
        gif.title,
        gif.preview_url,
        ratio,
        gif.send_url,
        gif.page_url,
    ]
}

/// `value` as Kotlin writes it: the shortest decimal that reads back as the same float, or
/// `NaN` and `Infinity`, which Rust spells otherwise.
fn float(value: f32) -> String {
    if value.is_nan() {
        "NaN".to_owned()
    } else if value.is_infinite() {
        if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned()
    } else {
        value.to_string()
    }
}

/// The URL that asks `endpoint` with these params, `names[i]` being `values[i]`'s, then the key
/// and GIPHY's settings.
pub fn url(
    api_key: &[u16],
    endpoint: &[u16],
    names: &[Vec<u16>],
    values: &[Vec<u16>],
) -> Result<Vec<u16>> {
    if names.len() != values.len() {
        return Err(illegal(format!(
            "{} names for {} values",
            names.len(),
            values.len()
        )));
    }
    let params: Vec<_> = names.iter().cloned().zip(values.iter().cloned()).collect();
    Ok(giphy::url(api_key, endpoint, &params))
}

#[cfg(test)]
mod tests {
    use super::float;

    #[test]
    fn floats_are_written_as_kotlin_reads_them() {
        assert_eq!(float(16.0 / 9.0), "1.7777778");
        assert_eq!(float(1.0), "1");
        assert_eq!(float(f32::NAN), "NaN");
        assert_eq!(float(f32::INFINITY), "Infinity");
        assert_eq!(float(f32::NEG_INFINITY), "-Infinity");
    }
}
