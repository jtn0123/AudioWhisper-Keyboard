//! The icons the Rust renderer draws, held here rather than sent from Kotlin.
//!
//! Each icon's source is the vector Kotlin drew it from: a Material icon from
//! `androidx.compose.material:material-icons-extended` 1.7.8 (Google's Material Design Icons,
//! Apache-2.0), or one of the app's vector drawables (Pictogrammers' Material Design Icons and
//! FlorisBoard's own, Apache-2.0). `data/sources.tsv` keeps each as Compose's path nodes;
//! `IconSourcesTest` (app/src/test) writes the file from those vectors and fails if it drifts.
//!
//! [`path`] turns a source into the text Kotlin's `iconPath` sent before: Compose's `toPath`
//! builds Skia's path, Android's `Path.approximate(0.25)` flattens it, and the points are written
//! in a 24-unit box. Each step is ported to the arithmetic, so `src/golden.rs` holds every
//! icon to the text an Android device made of it at 29cb06dcd, digit for digit.

mod approximate;
mod arc;
mod format;
#[cfg(test)]
mod golden;
mod nodes;
mod settings;
mod sites;
mod skia;
mod sources;

pub use settings::settings;
pub use sites::*;

use std::sync::OnceLock;

/// Icon `name`'s flattened path as `M x y L x y … Z` in a 24-unit box, or `None` for a name this
/// crate does not hold. Each icon is flattened once, when first asked for.
pub fn path(name: &str) -> Option<&'static str> {
    static FLAT: OnceLock<Vec<OnceLock<String>>> = OnceLock::new();
    let all = sources::all();
    let i = sources::index(name)?;
    let flat = FLAT.get_or_init(|| all.iter().map(|_| OnceLock::new()).collect());
    Some(flat[i].get_or_init(|| format::icon_path(&all[i])))
}

/// The drawn path of a key's icon `token` (see [`for_key`]), or "" for a token with none.
pub fn key_path(token: i32) -> &'static str {
    for_key(token).and_then(path).unwrap_or("")
}

/// Each of `names`' drawn paths, owned as the renderer's scenes hold them; "" for a name this
/// crate does not hold.
pub fn owned(names: &[&str]) -> Vec<String> {
    names
        .iter()
        .map(|n| path(n).unwrap_or_default().to_owned())
        .collect()
}

/// Every icon's name, in the order `data/sources.tsv` holds them.
pub fn names() -> impl Iterator<Item = &'static str> {
    sources::all().iter().map(|s| s.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_place_names_an_icon_this_crate_holds() {
        let mut named: Vec<&str> = EMOJI.into_iter().chain(CLIPBOARD).chain(SUBTYPES).collect();
        named.extend(SEARCH.into_iter().chain(ONE_HANDED.into_iter().flatten()));
        named.extend(settings::SETTINGS.iter().map(|(_, icon)| *icon));
        named.extend((0..12).filter_map(|i| settings(&format!("home{i}"))));
        named.extend((-10_000..2_000).filter_map(for_key));
        named.push(WATERMARK);
        for name in &named {
            assert!(path(name).is_some_and(|p| p.starts_with("M ")), "{name}");
        }
        // And every icon held is drawn somewhere.
        for name in names() {
            assert!(named.contains(&name), "{name} is drawn nowhere");
        }
    }

    #[test]
    fn every_source_reads() {
        let all = sources::all();
        assert_eq!(all.len(), names().count());
        for source in all {
            assert!(source.width > 0.0 && source.height > 0.0, "{}", source.name);
            for svg in &source.paths {
                assert!(nodes::parse(svg).is_some(), "{}", source.name);
            }
        }
    }
}
