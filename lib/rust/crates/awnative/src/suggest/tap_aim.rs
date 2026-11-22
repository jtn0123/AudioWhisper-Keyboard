//! The learned tap aim (`suggest::tap_aim` and `suggest::tap_learning`) on a `TapPositions`
//! handle (see tap.rs), for `SuggestNative`, whose generated entry points call these: the events
//! it learns from, the aimed offsets the decoder hears, the aimed hit test, and the saved text.
//! The text holds numbers only.

use jni_sys::jlong;
use keys::Rect;
use suggest::tap_aim::{AimKey, Orientation};
use suggest::tap_learning::Taps;

use super::tap::{packed, trail};
use crate::handle;
use crate::jni::{Result, illegal};
use crate::keys::{rect, rows};

/// The screen is in landscape (`true`) or portrait: taps are learned and aimed per orientation.
pub fn landscape(handle: i64, landscape: bool) -> Result<()> {
    trail(handle)?.set_orientation(Orientation::of(landscape));
    Ok(())
}

/// `word` was committed as typed (`Taps::kept`).
pub fn kept(handle: i64, word: &[u16]) -> Result<()> {
    trail(handle)?.kept(word);
    Ok(())
}

/// One of the events that confirm or throw away the word waiting to teach: 0 a backspace, 1 an
/// undo, 2 a suggestion tapped with nothing being typed, 3 one tapped for a word being typed, 4 a
/// swiped word.
pub fn event(handle: i64, event: i32) -> Result<()> {
    let mut taps = trail(handle)?;
    match event {
        0 => taps.backspace(),
        1 => taps.rejected(),
        2 => taps.picked(false),
        3 => taps.picked(true),
        4 => taps.swiped(),
        _ => return Err(illegal("unknown tap event")),
    }
    Ok(())
}

/// Where each character of `word` was tapped, moved by the learned aim if `aimed`
/// (`Taps::offsets_for`), packed as `tap::offsets_for` packs it.
pub fn aimed_offsets_for(handle: i64, word: &[u16], aimed: bool) -> Result<Vec<f32>> {
    Ok(packed(&trail(handle)?.offsets_for(word, aimed)))
}

/// Which key a tap at (`x`, `y`) types (`Taps::key_for`): moved by the learned aim if `aimed`, and
/// by the letter odds near a border while they hold. `layout` holds the key the plain hit test
/// found, the number of rows, each row's size, then each key's code; `bounds` holds six values per
/// key: its touch bounds (left, top, right, bottom) and its drawn width and height.
pub fn retarget(
    handle: i64,
    layout: &[i32],
    bounds: &[f32],
    x: f32,
    y: f32,
    aimed: bool,
) -> Result<i32> {
    let (&hit, rest) = layout.split_first().ok_or_else(|| illegal("no hit"))?;
    let (&row_count, rest) = rest.split_first().ok_or_else(|| illegal("no rows"))?;
    let row_count = usize::try_from(row_count).map_err(|_| illegal("negative rows"))?;
    let (row_sizes, codes) = rest
        .split_at_checked(row_count)
        .ok_or_else(|| illegal("fewer row sizes than rows"))?;
    let (bounds, []) = bounds.as_chunks::<6>() else {
        return Err(illegal("bounds needs six values per key"));
    };
    if codes.len() != bounds.len() {
        return Err(illegal("every key needs a code and bounds"));
    }
    let taps = trail(handle)?;
    let Ok(hit) = usize::try_from(hit) else {
        return Ok(hit);
    };
    let touch: Vec<Rect> = bounds
        .iter()
        .map(|&[left, top, right, bottom, ..]| rect(&[left, top, right, bottom]))
        .collect();
    let rows = rows(&touch, row_sizes)?;
    let keys: Vec<AimKey> = codes
        .iter()
        .zip(bounds)
        .map(|(&code, &[.., width, height])| AimKey {
            code,
            width,
            height,
        })
        .collect();
    let key_at = |x, y| keys::key_at(&rows, x, y);
    let priors = super::letter_odds::priors(&taps.trail, codes.iter().copied());
    let bounds = |index: usize| touch.get(index).copied();
    let key = taps.key_for(&keys, (hit, (x, y)), aimed, &priors, bounds, key_at);
    // The index fits in an Int: Kotlin's arrays held that many keys.
    Ok(key as i32)
}

/// What the Rust renderer's hit test leans on as a key goes down on `scene`: how far, in pixels,
/// this user's taps for each key land from where they mean, if `aimed` and that is known (see
/// `keys::aim::retarget`), and each key's letter odds while they hold (see `keys::resize`). Nothing
/// without a trail (`trail` 0).
///
/// # Safety
/// `trail` must be 0 or a live trail handle: its Kotlin owner is still reachable.
pub(crate) unsafe fn lean(
    trail: jlong,
    aimed: bool,
    scene: &mut render::scene::Scene,
) -> crate::jni::Result<()> {
    scene.aims.clear();
    scene.priors.clear();
    if trail == 0 {
        return Ok(());
    }
    // SAFETY: as the caller promises.
    let taps = unsafe { handle::locked::<Taps>(trail, "tap trail")? };
    let orientation = taps.orientation();
    if aimed {
        scene.aims = (scene.keys.iter())
            .map(|key| {
                let bias = taps.aim.bias(orientation, key.code)?;
                let size = key.bounds.visible;
                Some((bias.dx * size.width(), bias.dy * size.height()))
            })
            .collect();
    }
    scene.priors = super::letter_odds::priors(&taps.trail, scene.keys.iter().map(|key| key.code));
    Ok(())
}

/// The learned aim as saved text (`TapAim::text`): numbers only.
pub fn text(handle: i64) -> Result<Vec<u16>> {
    Ok(trail(handle)?.aim.text())
}

/// Replaces the learned aim with saved `text` (`Taps::load`).
pub fn load(handle: i64, text: &[u16]) -> Result<()> {
    trail(handle)?.load(text);
    Ok(())
}

/// Erases the learned aim (`Taps::forget`).
pub fn forget(handle: i64) -> Result<()> {
    trail(handle)?.forget();
    Ok(())
}

/// Whether the aim changed since this was last asked (`Taps::take_changed`).
pub fn changed(handle: i64) -> Result<bool> {
    Ok(trail(handle)?.take_changed())
}
