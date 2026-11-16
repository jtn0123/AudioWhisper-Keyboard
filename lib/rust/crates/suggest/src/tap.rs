//! Where each recently typed character was tapped, so the decoder hears that a tap on the O/P
//! border was nearly a P, not a clean O. Was Kotlin's `TapOffset` and `TapTrail`.

use std::collections::VecDeque;

use crate::java;

/// Where a tap landed in its key, from the key's centre, in key widths (`dx`, right is positive)
/// and key heights (`dy`, down is positive). A tap on the key's right edge is `dx` = 0.5.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TapOffset {
    pub dx: f32,
    pub dy: f32,
}

impl TapOffset {
    /// A tap in the gap beside a key still counts for it, but no further out than this.
    pub const LIMIT: f32 = 0.6;

    /// The offset of a tap at (`x`, `y`) on a key centred at (`center_x`, `center_y`), or `None`
    /// for a key with no size.
    pub fn of(
        x: f32,
        y: f32,
        center_x: f32,
        center_y: f32,
        width: f32,
        height: f32,
    ) -> Option<Self> {
        if width <= 0.0 || height <= 0.0 {
            return None;
        }
        Some(TapOffset {
            dx: coerce_in_limit((x - center_x) / width),
            dy: coerce_in_limit((y - center_y) / height),
        })
    }
}

/// Kotlin's `coerceIn(-LIMIT, LIMIT)`, which `clamp` matches, NaN included.
fn coerce_in_limit(value: f32) -> f32 {
    value.clamp(-TapOffset::LIMIT, TapOffset::LIMIT)
}

/// One typed character and where it was tapped, if it came from a tap.
#[derive(Clone, Copy, Debug)]
struct Typed {
    code_point: i32,
    offset: Option<TapOffset>,
}

/// The characters typed last and where each was tapped. The keyboard view reports each key press
/// ([`TapTrail::pressed`]); the text that press types is then [`TapTrail::typed`]. A character
/// with no press behind it (a long-press popup, a hardware key) has no position.
///
/// [`TapTrail::offsets_for`] lines the trail up with the word being suggested for from its end,
/// so the trail needs no resetting as words come and go: a word that doesn't match it (the cursor
/// moved, a whole word was deleted) simply gets no positions.
#[derive(Debug)]
pub struct TapTrail {
    capacity: usize,
    pressed_code_point: i32,
    pressed_offset: Option<TapOffset>,
    trail: VecDeque<Typed>,
    /// Characters typed since the last deletion (or since the trail was cleared).
    since_deletion: usize,
}

/// No key is pressed.
const NONE: i32 = -1;

impl TapTrail {
    /// Longer than any word worth correcting.
    pub const CAPACITY: usize = 64;

    /// A trail of the last `capacity` characters.
    pub fn new(capacity: usize) -> Self {
        TapTrail {
            capacity,
            pressed_code_point: NONE,
            pressed_offset: None,
            trail: VecDeque::new(),
            since_deletion: 0,
        }
    }

    /// A key for `code_point` went down at `offset`.
    pub fn pressed(&mut self, code_point: i32, offset: Option<TapOffset>) {
        self.pressed_code_point = code_point;
        self.pressed_offset = offset;
    }

    /// `text` went into the editor; its first character came from the last press if they match.
    pub fn typed(&mut self, text: &[u16]) {
        for (i, code_point) in java::code_points(text).into_iter().enumerate() {
            let from_press = i == 0 && same_key(code_point, self.pressed_code_point);
            let offset = if from_press {
                self.pressed_offset
            } else {
                None
            };
            self.trail.push_back(Typed { code_point, offset });
            self.since_deletion = self.since_deletion.saturating_add(1);
            while self.trail.len() > self.capacity {
                self.trail.pop_front();
            }
        }
        self.pressed_code_point = NONE;
        self.pressed_offset = None;
    }

    /// The last typed character was deleted.
    pub fn deleted(&mut self) {
        self.trail.pop_back();
        self.since_deletion = 0;
    }

    pub fn clear(&mut self) {
        self.trail.clear();
        self.since_deletion = 0;
        self.pressed_code_point = NONE;
        self.pressed_offset = None;
    }

    /// Where each character of `word` was tapped, if the last characters typed spell `word` (in
    /// any case) and at least one was tapped; otherwise nothing.
    pub fn offsets_for(&self, word: &[u16]) -> Vec<Option<TapOffset>> {
        let Some(typed) = self.typed_for(word) else {
            return Vec::new();
        };
        let offsets: Vec<_> = typed.iter().map(|&(_, offset)| offset).collect();
        if offsets.iter().all(Option::is_none) {
            return Vec::new();
        }
        offsets
    }

    /// The last characters typed, if they spell `word` (in any case): each as typed, with where
    /// it was tapped. `None` for an empty word or one the trail doesn't end with.
    pub fn typed_for(&self, word: &[u16]) -> Option<Vec<(i32, Option<TapOffset>)>> {
        let code_points = java::code_points(word);
        if code_points.is_empty() || code_points.len() > self.trail.len() {
            return None;
        }
        let start = self.trail.len() - code_points.len();
        let typed: Vec<_> = self
            .trail
            .range(start..)
            .map(|typed| (typed.code_point, typed.offset))
            .collect();
        let spells = typed
            .iter()
            .zip(&code_points)
            .all(|(&(typed, _), &cp)| same_key(typed, cp));
        spells.then_some(typed)
    }

    /// Whether the last `count` characters were all typed since the last deletion.
    pub fn typed_since_deletion(&self, count: usize) -> bool {
        count <= self.since_deletion
    }
}

fn same_key(a: i32, b: i32) -> bool {
    a == b || java::to_lower(a) == java::to_lower(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    const LEFT: TapOffset = TapOffset { dx: -0.4, dy: 0.0 };
    const RIGHT: TapOffset = TapOffset { dx: 0.45, dy: 0.1 };

    #[test]
    fn measures_a_tap_from_the_centre_in_keys() {
        let offset = TapOffset::of(145.0, 80.0, 100.0, 75.0, 100.0, 150.0).unwrap();
        assert_eq!((offset.dx, offset.dy), (0.45, 5.0 / 150.0));
        let far = TapOffset::of(500.0, -500.0, 0.0, 0.0, 100.0, 100.0).unwrap();
        assert_eq!((far.dx, far.dy), (0.6, -0.6));
        assert_eq!(TapOffset::of(1.0, 1.0, 0.0, 0.0, 0.0, 10.0), None);
        assert!(
            TapOffset::of(f32::NAN, 1.0, 0.0, 0.0, 1.0, 1.0)
                .unwrap()
                .dx
                .is_nan()
        );
    }

    #[test]
    fn lines_the_trail_up_with_the_word_from_its_end() {
        let mut trail = TapTrail::new(TapTrail::CAPACITY);
        trail.pressed('t' as i32, Some(LEFT));
        trail.typed(&utf16("t"));
        trail.typed(&utf16("e")); // no press: a popup
        trail.pressed('H' as i32, Some(RIGHT));
        trail.typed(&utf16("h"));
        assert_eq!(
            trail.offsets_for(&utf16("TEH")),
            [Some(LEFT), None, Some(RIGHT)]
        );
        assert_eq!(trail.offsets_for(&utf16("eh")), [None, Some(RIGHT)]);
        assert_eq!(trail.offsets_for(&utf16("e")), []);
        assert_eq!(trail.offsets_for(&utf16("ah")), []);
        assert_eq!(trail.offsets_for(&utf16("xteh")), []);
        trail.deleted();
        assert_eq!(trail.offsets_for(&utf16("te")), [Some(LEFT), None]);
        trail.clear();
        assert_eq!(trail.offsets_for(&utf16("te")), []);
    }

    #[test]
    fn a_press_counts_only_for_the_first_character_it_types() {
        let mut trail = TapTrail::new(TapTrail::CAPACITY);
        trail.pressed('a' as i32, Some(LEFT));
        trail.typed(&utf16("ab"));
        trail.pressed('x' as i32, Some(RIGHT));
        trail.typed(&utf16("c"));
        assert_eq!(trail.offsets_for(&utf16("abc")), [Some(LEFT), None, None]);
    }

    #[test]
    fn keeps_only_the_last_characters() {
        let mut trail = TapTrail::new(2);
        trail.pressed('a' as i32, Some(LEFT));
        trail.typed(&utf16("a"));
        trail.pressed('b' as i32, Some(RIGHT));
        trail.typed(&utf16("bc"));
        assert_eq!(trail.offsets_for(&utf16("abc")), []);
        assert_eq!(trail.offsets_for(&utf16("bc")), [Some(RIGHT), None]);
    }

    #[test]
    fn only_the_first_character_typed_comes_from_the_press() {
        let mut trail = TapTrail::new(TapTrail::CAPACITY);
        trail.pressed('a' as i32, Some(LEFT));
        trail.typed(&utf16("aa"));
        assert_eq!(trail.offsets_for(&utf16("aa")), vec![Some(LEFT), None]);
    }

    #[test]
    fn knows_what_was_typed_since_the_last_deletion() {
        let mut trail = TapTrail::new(TapTrail::CAPACITY);
        trail.pressed('a' as i32, Some(LEFT));
        trail.typed(&utf16("ab"));
        assert!(trail.typed_since_deletion(2));
        assert!(!trail.typed_since_deletion(3));
        trail.deleted();
        assert!(trail.typed_since_deletion(0));
        assert!(!trail.typed_since_deletion(1));
        trail.typed(&utf16("c"));
        assert_eq!(
            trail.typed_for(&utf16("AC")),
            Some(vec![('a' as i32, Some(LEFT)), ('c' as i32, None)])
        );
        assert!(trail.typed_since_deletion(1));
        assert!(!trail.typed_since_deletion(2));
        assert_eq!(trail.typed_for(&utf16("xc")), None);
        assert_eq!(trail.typed_for(&utf16("")), None);
        trail.clear();
        assert!(!trail.typed_since_deletion(1));
    }

    #[test]
    fn a_word_with_no_taps_gets_nothing() {
        let mut trail = TapTrail::new(TapTrail::CAPACITY);
        trail.typed(&utf16("ab"));
        assert!(trail.offsets_for(&utf16("ab")).is_empty());
    }

    #[test]
    fn a_press_matches_its_character_in_any_case() {
        // The Kelvin sign lower-cases to k.
        let mut trail = TapTrail::new(TapTrail::CAPACITY);
        trail.pressed(0x212A, Some(RIGHT));
        trail.typed(&utf16("k"));
        assert_eq!(trail.offsets_for(&utf16("K")), vec![Some(RIGHT)]);
    }
}
