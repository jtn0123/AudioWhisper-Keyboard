//! Kotlin's `LinkedHashMap` as the word stores use it: entries in the order they were added, where
//! setting a key that is already there leaves it in its place. A store moves an entry to the end
//! (most recently used) by removing it and adding it again, and drops the oldest from the front.

use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

#[derive(Clone, Debug)]
pub(crate) struct Ordered<K, V> {
    /// Each key's place in `entries`.
    places: HashMap<K, u64>,
    /// The entries by place, oldest first.
    entries: BTreeMap<u64, (K, V)>,
    /// The place of the next new key. 2⁶⁴ additions are out of reach.
    next: u64,
}

impl<K, V> Default for Ordered<K, V> {
    fn default() -> Self {
        Ordered {
            places: HashMap::new(),
            entries: BTreeMap::new(),
            next: 0,
        }
    }
}

impl<K: Clone + Eq + Hash, V> Ordered<K, V> {
    pub(crate) fn len(&self) -> usize {
        self.places.len()
    }

    pub(crate) fn contains_key<Q: Hash + Eq + ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.places.contains_key(key)
    }

    pub(crate) fn get<Q: Hash + Eq + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let place = self.places.get(key)?;
        self.entries.get(place).map(|(_, value)| value)
    }

    pub(crate) fn get_mut<Q: Hash + Eq + ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        let place = self.places.get(key)?;
        self.entries.get_mut(place).map(|(_, value)| value)
    }

    /// Kotlin's `map[key] = value`: a new key goes last, and one already here keeps its place.
    pub(crate) fn put(&mut self, key: K, value: V) {
        if let Some(place) = self.places.get(&key) {
            if let Some(entry) = self.entries.get_mut(place) {
                entry.1 = value;
            }
            return;
        }
        let place = self.next;
        self.next += 1;
        self.places.insert(key.clone(), place);
        self.entries.insert(place, (key, value));
    }

    pub(crate) fn remove<Q: Hash + Eq + ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        let place = self.places.remove(key)?;
        self.entries.remove(&place).map(|(_, value)| value)
    }

    pub(crate) fn clear(&mut self) {
        self.places.clear();
        self.entries.clear();
    }

    /// The entries, oldest first.
    pub(crate) fn iter(&self) -> impl DoubleEndedIterator<Item = (&K, &V)> {
        self.entries.values().map(|(key, value)| (key, value))
    }

    /// The entries, oldest first, taking them out.
    pub(crate) fn into_entries(self) -> impl Iterator<Item = (K, V)> {
        self.entries.into_values()
    }

    /// Drops the oldest entries until at most `capacity` are left: all of them, if it's negative.
    pub(crate) fn trim(&mut self, capacity: i32) {
        while i64::try_from(self.len()).unwrap_or(i64::MAX) > i64::from(capacity) {
            let Some((_, (key, _))) = self.entries.pop_first() else {
                break;
            };
            self.places.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(map: &Ordered<&'static str, i32>) -> Vec<(&'static str, i32)> {
        map.iter().map(|(&key, &value)| (key, value)).collect()
    }

    #[test]
    fn keeps_the_order_keys_were_added_in() {
        let mut map = Ordered::default();
        map.put("a", 1);
        map.put("b", 2);
        map.put("a", 3); // already here: keeps its place
        assert_eq!(keys(&map), [("a", 3), ("b", 2)]);
        let value = map.remove("a").unwrap();
        map.put("a", value + 1); // removed and added: now last
        assert_eq!(keys(&map), [("b", 2), ("a", 4)]);
        *map.get_mut("b").unwrap() = 5;
        assert_eq!((map.get("b"), map.get("c")), (Some(&5), None));
        assert!(map.contains_key("a") && !map.contains_key("c"));
        assert_eq!(map.iter().next_back(), Some((&"a", &4)));
    }

    #[test]
    fn trims_from_the_oldest() {
        let mut map = Ordered::default();
        for (i, key) in ["a", "b", "c", "d"].into_iter().enumerate() {
            map.put(key, i as i32);
        }
        map.trim(2);
        assert_eq!(keys(&map), [("c", 2), ("d", 3)]);
        map.trim(5);
        assert_eq!(map.len(), 2);
        map.trim(-1);
        assert_eq!(map.len(), 0);
        map.put("e", 4);
        assert_eq!(map.into_entries().collect::<Vec<_>>(), [("e", 4)]);
    }
}
