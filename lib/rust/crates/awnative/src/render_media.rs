//! Decoded pixel upload; no borrowed JVM buffer escapes this boundary.
use crate::{handle, jni::illegal, render::Session};
use jni_sys::{JNIEnv, jboolean, jclass, jint, jintArray, jlong, jlongArray};
use std::ptr;
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_mediaRequests(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
) -> jlongArray {
    // SAFETY: synchronized wrapper owns the session; ids are returned in a new local JVM array.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            env.make(
                &s.scene
                    .panel
                    .as_ref()
                    .map_or(vec![], |p| p.media.requests()),
            )
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_mediaUpload(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    id: jlong,
    index: jint,
    width: jint,
    height: jint,
    delay: jlong,
    last: jboolean,
    pixels: jintArray,
) -> jboolean {
    // SAFETY: JVM owns pixels during this call; copied pixels are validated before raster use.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", 0, |s, env| {
            if width <= 0 || height <= 0 || width > 1024 || height > 1024 || index < 0 {
                return Err(illegal("media dimensions"));
            }
            let pixels = env.read::<i32>(pixels, "media pixels")?;
            let Some(p) = &mut s.scene.panel else {
                return Ok(0);
            };
            // An admitted decode can finish after scrolling away; only visible entries tick.
            if !p.media.visible.contains(&id) && (index == 0 || !p.media.entries.contains_key(&id))
            {
                return Ok(0);
            }
            let Some(frame) =
                render::media_cache::Frame::argb(width as u16, height as u16, &pixels, delay)
            else {
                return Err(illegal("media pixels"));
            };
            let accepted = p.media.upload(id, index as usize, frame, last != 0);
            if accepted && let Some(c) = &mut p.clipboard {
                for e in &mut c.entries {
                    if e.id == id {
                        e.aspect = width as f32 / height as f32;
                    }
                }
            }
            s.prepare_panel();
            Ok(u8::from(accepted))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_mediaRepeat(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    id: jlong,
    count: jint,
) {
    // SAFETY: synchronized wrapper retains the session; repeat metadata contains no borrowed memory.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, _| {
            if let Some(p) = &mut s.scene.panel {
                p.media.repeat(id, count);
            }
            Ok(())
        });
    }
}
