//! Input-only JNI boundary. Native owns all pointer, swipe and glide recognition state.
use crate::{
    jni::{Result, illegal},
    render::{session, texts},
};
use render::{gesture_config::GestureConfig, touch::Motion};
pub fn gestures(native: i64, values: &[f32], actions: &[Vec<u16>]) -> Result<()> {
    session(native, |s| {
        let config = GestureConfig::decode(values, texts(actions))
            .ok_or_else(|| illegal("invalid gesture settings"))?;
        s.touch.configure(config);
        Ok(())
    })
}
pub fn long_press(native: i64, pointer: i32, popup: bool, uninterrupted: bool) -> Result<Vec<i32>> {
    session(native, |s| {
        let actions: Vec<_> = s
            .touch
            .long_press(&s.scene, pointer, popup, uninterrupted)
            .into_iter()
            .flat_map(|a| [a.0, a.1, a.2])
            .collect();
        s.prepare_panel();
        Ok(actions)
    })
}
pub fn touch(
    native: i64,
    metadata: &[i32],
    times: &[i64],
    coords: &[f32],
    trail: i64,
    aimed: bool,
) -> Result<Vec<i32>> {
    session(native, |s| {
        let meta = metadata;
        if meta.len() < 2 {
            return Err(illegal("missing action/index"));
        }
        // A key goes down: where this user aims and which letters come next, as of now
        // (`aimed_key_at`).
        if matches!(meta[0], 0 | 5) {
            // SAFETY: `trail` is 0 or a tap trail its Kotlin owner keeps reachable while it calls.
            unsafe { crate::suggest::tap_aim::lean(trail, aimed, &mut s.scene)? };
        }
        let event = Motion {
            action: meta[0],
            index: meta[1] as usize,
            ids: meta[2..].to_vec(),
            times: times.to_vec(),
            coords: coords.to_vec(),
        };
        s.touch_time = event.times.last().copied().unwrap_or(0);
        let actions: Vec<_> = s
            .touch
            .event(&mut s.scene, &event)
            .into_iter()
            .flat_map(|a| [a.0, a.1, a.2])
            .collect();
        s.style_bubble(false);
        s.prepare_panel();
        Ok(actions)
    })
}
