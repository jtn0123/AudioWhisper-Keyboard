use super::*;
use crate::utf16;

#[test]
fn a_search_is_kept_by_its_trimmed_lower_case() {
    assert_eq!(key(&utf16(" Cats\t")), utf16("cats"));
    assert_eq!(key(&utf16("İ")), lowercase(&utf16("İ")));
    let mut cache = ResultCache::default();
    assert!(cache.put(&key(&utf16("Cats")), 0).is_empty());
    assert!(cache.get(&key(&utf16("cats ")), 1));
    assert!(!cache.get(&key(&utf16("dogs")), 1));
}

#[test]
fn a_search_lasts_ten_minutes() {
    let mut cache = ResultCache::default();
    cache.put(&utf16("cats"), 1_000);
    assert!(cache.get(&utf16("cats"), 1_000 + TTL_MS - 1));
    assert!(!cache.get(&utf16("cats"), 1_000 + TTL_MS));
    // Once stale it is gone, even if the clock went back.
    assert!(!cache.get(&utf16("cats"), 1_000));
    // Time wraps as Kotlin's Long does: far in the past is the future.
    cache.put(&utf16("dogs"), i64::MAX);
    assert!(cache.get(&utf16("dogs"), i64::MIN));
}

#[test]
fn the_least_recently_used_goes_when_full() {
    let mut cache = ResultCache::new(TTL_MS, 2);
    cache.put(&utf16("a"), 0);
    cache.put(&utf16("b"), 0);
    assert!(cache.get(&utf16("a"), 0));
    assert_eq!(cache.put(&utf16("c"), 0), [utf16("b")]);
    // Storing again counts as using it.
    cache.put(&utf16("a"), 0);
    assert_eq!(cache.put(&utf16("d"), 0), [utf16("c")]);
    // With no room, what was stored goes at once.
    assert_eq!(
        ResultCache::new(TTL_MS, 0).put(&utf16("a"), 0),
        [utf16("a")]
    );
}
