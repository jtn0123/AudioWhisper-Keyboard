//! Which searches the GIF panel's result cache keeps, as Kotlin's `GifResultCache` had it: each
//! search's GIFs for [`TTL_MS`], at most [`CAPACITY`] searches, the least recently used going
//! first. The GIFs themselves stay in Kotlin, under the [`key`] this gives.

use chars::{lowercase, trim};

/// How long a search's GIFs are kept, in milliseconds: trending changes, and a search may have
/// new GIFs.
pub const TTL_MS: i64 = 10 * 60 * 1000;

/// How many searches are kept.
pub const CAPACITY: usize = 20;

/// The key `search` is kept under: "Cats" and "cats " are the same search.
pub fn key(search: &[u16]) -> Vec<u16> {
    lowercase(trim(search))
}

/// The keys kept and when each was stored, least recently used first.
#[derive(Debug)]
pub struct ResultCache {
    ttl_ms: i64,
    capacity: usize,
    entries: Vec<(Vec<u16>, i64)>,
}

impl ResultCache {
    pub fn new(ttl_ms: i64, capacity: usize) -> Self {
        ResultCache {
            ttl_ms,
            capacity,
            entries: Vec::new(),
        }
    }

    /// Whether `key`'s GIFs, stored before, are still kept at `now`. Asking counts as using it;
    /// a stale one is dropped. Kotlin's `Long` time wraps.
    pub fn get(&mut self, key: &[u16], now: i64) -> bool {
        let Some(at) = self.entries.iter().position(|(it, _)| it == key) else {
            return false;
        };
        let entry = self.entries.remove(at);
        if now.wrapping_sub(entry.1) >= self.ttl_ms {
            return false;
        }
        self.entries.push(entry);
        true
    }

    /// `key`'s GIFs were stored at `now`: the keys dropped to make room, eldest first.
    pub fn put(&mut self, key: &[u16], now: i64) -> Vec<Vec<u16>> {
        self.entries.retain(|(it, _)| it != key);
        self.entries.push((key.to_vec(), now));
        let over = self.entries.len().saturating_sub(self.capacity);
        self.entries.drain(..over).map(|(it, _)| it).collect()
    }
}

impl Default for ResultCache {
    fn default() -> Self {
        ResultCache::new(TTL_MS, CAPACITY)
    }
}

#[cfg(test)]
mod tests;
