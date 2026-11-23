//! Which previews the GIF panel keeps in memory while it is open, as Android's `LruCache` kept
//! them for Kotlin's `GifImages`: up to [`CAPACITY_BYTES`] of them, the least recently used
//! going first. The bytes stay in Kotlin, under the URLs this keeps.

/// How many bytes of previews are kept.
pub const CAPACITY_BYTES: usize = 24 * 1024 * 1024;

/// The URLs kept and their sizes, least recently used first.
#[derive(Debug)]
pub struct Previews {
    capacity: usize,
    size: usize,
    entries: Vec<(Vec<u16>, usize)>,
}

impl Default for Previews {
    fn default() -> Self {
        Previews::new(CAPACITY_BYTES)
    }
}

impl Previews {
    pub fn new(capacity: usize) -> Self {
        Previews {
            capacity,
            size: 0,
            entries: Vec::new(),
        }
    }

    /// Whether `url` is kept; if so, it becomes the most recently used.
    pub fn get(&mut self, url: &[u16]) -> bool {
        let Some(at) = self.entries.iter().position(|(kept, _)| kept == url) else {
            return false;
        };
        let entry = self.entries.remove(at);
        self.entries.push(entry);
        true
    }

    /// `url`'s preview of `bytes` is kept, replacing any before it, as the most recently used:
    /// the URLs dropped to make room, which may include `url` itself if it alone is too large.
    pub fn put(&mut self, url: &[u16], bytes: usize) -> Vec<Vec<u16>> {
        if let Some(at) = self.entries.iter().position(|(kept, _)| kept == url) {
            let (_, before) = self.entries.remove(at);
            self.size -= before;
        }
        self.entries.push((url.to_vec(), bytes));
        self.size += bytes;
        self.trim(self.capacity)
    }

    /// Drops every preview.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.size = 0;
    }

    fn trim(&mut self, capacity: usize) -> Vec<Vec<u16>> {
        let mut dropped = Vec::new();
        while self.size > capacity && !self.entries.is_empty() {
            let (url, bytes) = self.entries.remove(0);
            self.size -= bytes;
            dropped.push(url);
        }
        dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utf16;

    #[test]
    fn the_least_recently_used_go_first() {
        let mut previews = Previews::new(10);
        assert!(previews.put(&utf16("a"), 4).is_empty());
        assert!(previews.put(&utf16("b"), 4).is_empty());
        assert!(previews.get(&utf16("a")));
        assert_eq!(previews.put(&utf16("c"), 4), vec![utf16("b")]);
        assert!(!previews.get(&utf16("b")));
        assert!(previews.get(&utf16("a")));
        assert!(previews.get(&utf16("c")));
    }

    #[test]
    fn a_replaced_preview_counts_once_and_one_too_large_drops_itself() {
        let mut previews = Previews::new(10);
        previews.put(&utf16("a"), 6);
        assert!(previews.put(&utf16("a"), 9).is_empty());
        assert_eq!(previews.put(&utf16("b"), 2), vec![utf16("a")]);
        assert_eq!(
            previews.put(&utf16("big"), 11),
            vec![utf16("b"), utf16("big")]
        );
        assert!(previews.put(&utf16("z"), 0).is_empty());
        previews.clear();
        assert!(!previews.get(&utf16("z")));
        assert!(previews.put(&utf16("y"), 10).is_empty());
        assert_eq!(Previews::default().capacity, 25_165_824);
    }
}
