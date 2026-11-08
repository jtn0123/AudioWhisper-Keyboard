//! The keys of the layout a swipe was drawn on.

use crate::gesture::{distance, java_min};

/// A key on the layout: its code, and the centre and size of its visible bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key {
    pub code: i32,
    pub center_x: f32,
    pub center_y: f32,
    pub width: f32,
    pub height: f32,
}

/// A layout's keys, in layout order, matched to the key codes an index uses.
#[derive(Debug)]
pub struct Layout<'a> {
    keys: &'a [Key],
    /// The key for each of the index's codes: the last one with that code, as the keyboard's
    /// code-to-key map keeps it; none if the layout lacks it.
    slots: Vec<Option<Key>>,
}

impl<'a> Layout<'a> {
    /// `keys` as `codes` (from [`crate::GlideIndex::codes`]) find them.
    pub fn new(keys: &'a [Key], codes: &[i32]) -> Self {
        let slots = codes
            .iter()
            .map(|&code| keys.iter().rev().find(|key| key.code == code).copied())
            .collect();
        Layout { keys, slots }
    }

    pub(crate) fn slot(&self, slot: usize) -> Option<&Key> {
        self.slots.get(slot).and_then(Option::as_ref)
    }

    /// The first key's shorter side, which sets how far a swipe can stray; none without keys.
    pub(crate) fn radius(&self) -> Option<f32> {
        self.keys.first().map(|key| java_min(key.height, key.width))
    }

    /// The codes of the (up to) two keys whose centres are nearest (x, y), nearest first. Equally
    /// near keys keep layout order.
    pub(crate) fn closest_two(&self, x: f32, y: f32) -> Vec<i32> {
        let mut best: Vec<(f32, i32)> = Vec::with_capacity(3);
        for key in self.keys {
            let d = distance(key.center_x, key.center_y, x, y);
            let at = best
                .iter()
                .position(|&(other, _)| java_less(d, other))
                .unwrap_or(best.len());
            if at < 2 {
                best.insert(at, (d, key.code));
                best.truncate(2);
            }
        }
        best.into_iter().map(|(_, code)| code).collect()
    }
}

/// Whether `a` sorts before `b` as Java's `Float.compare` orders them: NaN after everything, and
/// -0 before +0.
fn java_less(a: f32, b: f32) -> bool {
    if a.is_nan() {
        false
    } else if b.is_nan() {
        true
    } else {
        a < b || (a == b && a.is_sign_negative() && b.is_sign_positive())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: i32, x: f32, y: f32) -> Key {
        Key {
            code,
            center_x: x,
            center_y: y,
            width: 10.0,
            height: 8.0,
        }
    }

    #[test]
    fn the_nearest_two_keys_come_nearest_first() {
        let keys = [
            key(1, 0.0, 0.0),
            key(2, 10.0, 0.0),
            key(3, 20.0, 0.0),
            key(4, 30.0, 0.0),
        ];
        let layout = Layout::new(&keys, &[]);

        assert_eq!(layout.closest_two(19.0, 1.0), vec![3, 2]);
        assert_eq!(layout.closest_two(-50.0, 0.0), vec![1, 2]);
        assert_eq!(layout.closest_two(100.0, 0.0), vec![4, 3]);
    }

    #[test]
    fn equally_near_keys_keep_layout_order() {
        let keys = [key(1, 0.0, 0.0), key(2, 10.0, 0.0), key(3, 20.0, 0.0)];
        let layout = Layout::new(&keys, &[]);

        assert_eq!(layout.closest_two(10.0, 0.0), vec![2, 1]);
        assert_eq!(layout.closest_two(5.0, 0.0), vec![1, 2]);
    }

    #[test]
    fn one_key_is_all_there_is() {
        let keys = [key(7, 0.0, 0.0)];

        assert_eq!(Layout::new(&keys, &[]).closest_two(3.0, 3.0), vec![7]);
        assert!(Layout::new(&[], &[]).closest_two(3.0, 3.0).is_empty());
    }

    #[test]
    fn a_code_finds_its_last_key() {
        let keys = [key(1, 0.0, 0.0), key(2, 10.0, 0.0), key(1, 20.0, 0.0)];
        let layout = Layout::new(&keys, &[1, 9, 2]);

        assert_eq!(layout.slot(0).map(|k| k.center_x), Some(20.0));
        assert_eq!(layout.slot(1), None);
        assert_eq!(layout.slot(2).map(|k| k.center_x), Some(10.0));
        assert_eq!(layout.slot(3), None);
    }

    #[test]
    fn the_radius_is_the_first_keys_shorter_side() {
        let keys = [
            key(1, 0.0, 0.0),
            Key {
                code: 2,
                center_x: 0.0,
                center_y: 0.0,
                width: 2.0,
                height: 2.0,
            },
        ];

        assert_eq!(Layout::new(&keys, &[]).radius(), Some(8.0));
        assert_eq!(Layout::new(&[], &[]).radius(), None);
    }

    #[test]
    fn java_ordering_puts_nan_last_and_negative_zero_first() {
        assert!(java_less(1.0, f32::NAN));
        assert!(!java_less(f32::NAN, 1.0));
        assert!(!java_less(f32::NAN, f32::NAN));
        assert!(java_less(-0.0, 0.0));
        assert!(!java_less(0.0, -0.0));
        assert!(!java_less(2.0, 2.0));
    }
}
