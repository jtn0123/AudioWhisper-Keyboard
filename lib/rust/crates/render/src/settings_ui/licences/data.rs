//! AboutLibraries 15.3.0's `Libs.Builder.withJson(...).build()` over the generated
//! `aboutlibraries.json`, as `AndroidParser` reads it with org.json: the licences, each
//! library's licence set in `HashSet` order, the author line, and the `name.lowercase()` order.
use serde_json::{Map, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct Licence {
    /// The `licenses` object's key: `License.hash`, what equality and `hashCode` use.
    pub hash: String,
    pub name: String,
    /// `licenseContent` (the JSON's `content`), or None.
    pub content: Option<String>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    pub id: String,
    pub name: String,
    /// `artifactVersion`, or None.
    pub version: Option<String>,
    /// `Library.author`: the developers' names joined, else the organisation's name.
    pub author: String,
    /// Indexes into [`Libraries::licences`], in the iteration order of the library's HashSet.
    pub licences: Vec<usize>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Libraries {
    pub libraries: Vec<Library>,
    pub licences: Vec<Licence>,
}
impl Libraries {
    /// `Library.htmlReadyLicenseContent`: each licence's content with `\n` as `<br />`,
    /// joined by four line breaks.
    pub fn html(&self, library: &Library) -> String {
        library
            .licences
            .iter()
            .map(|&i| {
                self.licences[i]
                    .content
                    .as_deref()
                    .unwrap_or("")
                    .replace('\n', "<br />")
            })
            .collect::<Vec<_>>()
            .join("<br /><br /><br /><br />")
    }
}

/// org.json's `opt(name)` as a string: absent and JSON null are None, anything else its text.
fn opt_string(o: &Map<String, Value>, name: &str) -> Option<String> {
    match o.get(name)? {
        Value::Null => None,
        v => Some(text(v)),
    }
}
/// org.json's `toString` of a value: strings as they are, numbers and the rest as JSON.
fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".into(),
        v => v.to_string(),
    }
}
/// `getString`: present (JSON null reads "null"), else the parse fails.
fn get_string(o: &Map<String, Value>, name: &str) -> Option<String> {
    o.get(name).map(text)
}
/// `optString(name, fallback)`: the fallback only when absent.
fn opt_string_or(o: &Map<String, Value>, name: &str, fallback: &str) -> String {
    o.get(name).map_or_else(|| fallback.to_owned(), text)
}

/// Java's `String.hashCode` over UTF-16 units.
pub fn java_hash(s: &str) -> i32 {
    s.encode_utf16()
        .fold(0i32, |h, c| h.wrapping_mul(31).wrapping_add(i32::from(c)))
}
/// Kotlin `toHashSet()` on a list of `n`: `HashSet(mapCapacity(n))`, whose table is the
/// power of two `tableSizeFor` picks; elements iterate by bucket, then insertion order.
fn hash_set_order(hashes: &[String]) -> Vec<usize> {
    let n = hashes.len();
    let capacity = if n < 3 {
        n + 1
    } else {
        (n as f32 / 0.75 + 1.0) as usize
    };
    let table = capacity.max(1).next_power_of_two().min(1 << 30);
    let mut seen: Vec<&str> = vec![];
    let mut order: Vec<(usize, usize, usize)> = vec![];
    for (i, hash) in hashes.iter().enumerate() {
        if seen.contains(&hash.as_str()) {
            continue;
        }
        seen.push(hash);
        let h = java_hash(hash) as u32;
        let bucket = ((h ^ (h >> 16)) as usize) & (table - 1);
        order.push((bucket, order.len(), i));
    }
    order.sort();
    order.into_iter().map(|(_, _, i)| i).collect()
}
/// Kotlin's `String.compareTo`: UTF-16 unit order.
fn utf16_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

/// `parseData`: any structural failure yields no libraries and no licences, as AndroidParser's
/// catch-all does.
pub fn parse(json: &str) -> Libraries {
    parse_strict(json).unwrap_or_default()
}
fn parse_strict(json: &str) -> Option<Libraries> {
    let root: Value = serde_json::from_str(json).ok()?;
    let root = root.as_object()?;
    let mut licences = vec![];
    for (key, value) in root.get("licenses")?.as_object()? {
        let o = value.as_object()?;
        licences.push(Licence {
            hash: key.clone(),
            name: get_string(o, "name")?,
            content: opt_string(o, "content"),
        });
    }
    let mut libraries = vec![];
    for value in root.get("libraries")?.as_array()? {
        let o = value.as_object()?;
        let ids: Vec<String> = match o.get("licenses") {
            Some(Value::Array(a)) => a.iter().map(text).collect(),
            _ => vec![],
        };
        // `mappedLicenses[this]` then `mapNotNull`: unknown keys drop before the set is built.
        let known: Vec<String> = ids
            .into_iter()
            .filter(|id| licences.iter().any(|l: &Licence| &l.hash == id))
            .collect();
        let set: Vec<usize> = hash_set_order(&known)
            .into_iter()
            .map(|i| {
                licences
                    .iter()
                    .position(|l| l.hash == known[i])
                    .expect("filtered to known licences")
            })
            .collect();
        let developers: Vec<Option<String>> = match o.get("developers") {
            Some(Value::Array(a)) => a
                .iter()
                .map(|d| d.as_object().map(|d| opt_string(d, "name")))
                .collect::<Option<_>>()?,
            _ => vec![],
        };
        let organisation = match o.get("organization") {
            Some(Value::Object(org)) => Some(opt_string_or(org, "name", "")),
            _ => None,
        };
        if let Some(Value::Array(funding)) = o.get("funding") {
            for f in funding {
                let f = f.as_object()?;
                get_string(f, "platform")?;
                get_string(f, "url")?;
            }
        }
        let id = get_string(o, "uniqueId")?;
        let author = if developers.is_empty() {
            organisation.unwrap_or_default()
        } else {
            developers
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(", ")
        };
        libraries.push(Library {
            name: opt_string_or(o, "name", &id),
            id,
            version: opt_string(o, "artifactVersion"),
            author,
            licences: set,
        });
    }
    // `sortedBy { it.name.lowercase() }`: a stable sort on the lowercased names.
    let keys: Vec<String> = libraries.iter().map(|l| l.name.to_lowercase()).collect();
    let mut order: Vec<usize> = (0..libraries.len()).collect();
    order.sort_by(|&a, &b| utf16_cmp(&keys[a], &keys[b]));
    let libraries = order.into_iter().map(|i| libraries[i].clone()).collect();
    Some(Libraries {
        libraries,
        licences,
    })
}
