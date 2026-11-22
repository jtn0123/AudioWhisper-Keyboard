//! Decoded pixel upload; no borrowed JVM buffer escapes this boundary.
use crate::{
    boundary::args::RenderNativeMediaUpload,
    jni::{Result, illegal},
    render::session,
};
pub fn media_requests(native: i64) -> Result<Vec<i64>> {
    session(native, |s| {
        Ok(s.scene
            .panel
            .as_ref()
            .map_or(vec![], |p| p.media.requests()))
    })
}
pub fn media_upload(args: RenderNativeMediaUpload) -> Result<bool> {
    let RenderNativeMediaUpload {
        handle,
        id,
        index,
        width,
        height,
        delay,
        last,
        pixels,
    } = args;
    session(handle, |s| {
        if width <= 0 || height <= 0 || width > 1024 || height > 1024 || index < 0 {
            return Err(illegal("media dimensions"));
        }
        let Some(p) = &mut s.scene.panel else {
            return Ok(false);
        };
        // An admitted decode can finish after scrolling away; only visible entries tick.
        if !p.media.visible.contains(&id) && (index == 0 || !p.media.entries.contains_key(&id)) {
            return Ok(false);
        }
        let Some(frame) =
            render::media_cache::Frame::argb(width as u16, height as u16, pixels, delay)
        else {
            return Err(illegal("media pixels"));
        };
        let accepted = p.media.upload(id, index as usize, frame, last);
        if accepted && let Some(c) = &mut p.clipboard {
            for e in &mut c.entries {
                if e.id == id {
                    e.aspect = width as f32 / height as f32;
                }
            }
        }
        s.prepare_panel();
        Ok(accepted)
    })
}

pub fn media_repeat(native: i64, id: i64, count: i32) -> Result<()> {
    session(native, |s| {
        if let Some(p) = &mut s.scene.panel {
            p.media.repeat(id, count);
        }
        Ok(())
    })
}
