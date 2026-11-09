use super::*;
use crate::utf16;

fn pick(assets: &[&str], language: &str, country: &str, variant: &str) -> Option<usize> {
    let assets: Vec<Vec<u16>> = assets.iter().map(|a| utf16(a)).collect();
    let assets: Vec<&[u16]> = assets.iter().map(Vec::as_slice).collect();
    resolve(&assets, &utf16(language), &utf16(country), &utf16(variant))
}

#[test]
fn the_most_specific_file_wins() {
    let assets = [
        "de.txt",
        "en.txt",
        "en_US.txt",
        "en_US_POSIX.txt",
        "root.txt",
    ];
    assert_eq!(pick(&assets, "en", "US", "POSIX"), Some(3));
    assert_eq!(pick(&assets, "en", "US", ""), Some(2));
    assert_eq!(pick(&assets, "en", "GB", "POSIX"), Some(1));
    assert_eq!(pick(&assets, "EN", "US", " "), Some(2));
    assert_eq!(pick(&assets, "en", " ", "POSIX"), Some(1));
    assert_eq!(pick(&assets, "en", "us", ""), Some(1));
    assert_eq!(pick(&assets, "fr", "FR", ""), None);
    assert_eq!(pick(&assets, "", "", ""), None);
    assert_eq!(pick(&[".txt"], "", "", ""), Some(0));
}

#[test]
fn a_variant_needs_a_country() {
    assert_eq!(
        pick(&["en__POSIX.txt", "en.txt"], "en", "", "POSIX"),
        Some(1)
    );
}
