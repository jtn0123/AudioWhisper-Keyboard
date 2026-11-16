//! Which key an ambiguous tap types, for a user whose taps for each key are known to land a little
//! off its centre (`suggest::tap_aim` learns this). Both renderers' hit tests go through here.

/// The key a tap at (`x`, `y`) types, given that the plain hit test (`key_at`) found `hit`, and
/// `shift(i)` is how far this user's taps for key `i` land from where they mean, in pixels, if
/// that is known.
///
/// The tap moves to a neighbour only when the hit key's shift says it was meant for that neighbour
/// and the neighbour's own shift agrees: taken back by the neighbour's shift, the tap still lands
/// on the neighbour. Without a shift for both keys, or away from a border, this is `hit`.
pub fn retarget(
    hit: usize,
    (x, y): (f32, f32),
    shift: impl Fn(usize) -> Option<(f32, f32)>,
    key_at: impl Fn(f32, f32) -> Option<usize>,
) -> usize {
    let meant_by = |key: usize| shift(key).map(|(dx, dy)| (x - dx, y - dy));
    let Some((hx, hy)) = meant_by(hit) else {
        return hit;
    };
    let Some(meant) = key_at(hx, hy).filter(|&key| key != hit) else {
        return hit;
    };
    match meant_by(meant) {
        Some((mx, my)) if key_at(mx, my) == Some(meant) => meant,
        _ => hit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three keys ten wide in a row.
    fn key_at(x: f32, _: f32) -> Option<usize> {
        let column = (x / 10.0).floor();
        (0.0..3.0).contains(&column).then_some(column as usize)
    }

    #[test]
    fn moves_a_tap_only_when_both_keys_agree() {
        let short = |_: usize| Some((-2.0, 0.0));
        assert_eq!(retarget(0, (9.0, 0.0), short, key_at), 1);
        assert_eq!(retarget(0, (5.0, 0.0), short, key_at), 0);
        assert_eq!(retarget(1, (19.5, 0.0), short, key_at), 2);
        // Key 1's taps land on its centre.
        let only_first = |key: usize| Some(if key == 0 { (-2.0, 0.0) } else { (0.0, 0.0) });
        assert_eq!(retarget(0, (9.0, 0.0), only_first, key_at), 0);
        // Key 1's aim is not known.
        let unknown = |key: usize| (key == 0).then_some((-2.0, 0.0));
        assert_eq!(retarget(0, (9.0, 0.0), unknown, key_at), 0);
        // Nothing is known, or nothing is off.
        assert_eq!(retarget(0, (9.9, 0.0), |_| None, key_at), 0);
        assert_eq!(retarget(0, (9.9, 0.0), |_| Some((0.0, 0.0)), key_at), 0);
        // Off the keys.
        assert_eq!(retarget(2, (29.0, 0.0), short, key_at), 2);
    }
}
