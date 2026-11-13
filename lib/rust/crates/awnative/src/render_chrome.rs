//! Offscreen Vello CPU drawing for small chrome views and settings images.
use crate::{
    handle,
    jni::{self, illegal},
    render::{Session, strings},
};
use jni_sys::{JNIEnv, jclass, jfloatArray, jintArray, jlong, jobjectArray};
use std::ptr;
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_chrome(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    config: jfloatArray,
    labels: jobjectArray,
    icons: jobjectArray,
) {
    // SAFETY: synchronized JVM owner retains the session; inputs are copied before drawing.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let c = env.read::<f32>(config, "chrome config")?;
            if c.len() != 4
                || c.iter().any(|v| !v.is_finite())
                || !(1.0..=8192.0).contains(&c[0])
                || !(1.0..=8192.0).contains(&c[1])
                || c[2] <= 0.0
            {
                return Err(illegal("chrome dimensions"));
            }
            if s.scene.width != c[0] as u16 || s.scene.height != c[1] as u16 {
                s.bitmap = None;
            }
            s.scene.width = c[0] as u16;
            s.scene.height = c[1] as u16;
            s.scene.strip = 0.0;
            s.scene.panel = Some(render::panels::Panel {
                kind: 8,
                density: c[2],
                chrome: Some(render::ime_chrome::Chrome {
                    kind: c[3] as i32,
                    labels: strings(env, labels)?,
                    icons: strings(env, icons)?,
                }),
                ..Default::default()
            });
            s.prepare_panel();
            Ok(())
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_pixels(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
) -> jintArray {
    // SAFETY: synchronized JVM owner retains the session; returned pixels are a fresh primitive array.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            s.prepare_panel();
            let raster = s.bitmap.get_or_insert_with(|| {
                render::raster::Raster::new(s.scene.width, s.scene.height, s.fonts.clone())
            });
            raster
                .draw(
                    &s.scene,
                    keys::Rect {
                        left: 0.0,
                        top: 0.0,
                        right: s.scene.width as f32,
                        bottom: s.scene.height as f32,
                    },
                )
                .map_err(|_| illegal("chrome raster failed"))?;
            let pixels: Vec<i32> = raster
                .pixels()
                .chunks_exact(4)
                .map(|p| {
                    let a = p[3] as u32;
                    let channel = |v: u8| {
                        if a == 0 {
                            0
                        } else {
                            ((v as u32 * 255 + a / 2) / a).min(255)
                        }
                    };
                    ((a << 24) | (channel(p[0]) << 16) | (channel(p[1]) << 8) | channel(p[2]))
                        as i32
                })
                .collect();
            env.make(&pixels)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ImeChromeRules_oneHanded(
    env: *mut JNIEnv,
    _: jclass,
    config: jintArray,
) -> jfloatArray {
    // SAFETY: local array is copied, and the output is a fresh JVM-owned array.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<i32>(config, "one-handed config")?;
            if c.len() != 4 {
                return Err(illegal("one-handed config"));
            }
            env.make(&render::ime_chrome::one_handed(
                c[0] != 0,
                c[1] != 0,
                c[2],
                c[3] != 0,
            ))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ImeChromeRules_sideButtons(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
) -> jfloatArray {
    // SAFETY: local array is copied and no reference escapes Rust.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<f32>(config, "side dimensions")?;
            if c.len() != 2 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
                return Err(illegal("side dimensions"));
            }
            env.make(
                &render::ime_chrome::side_buttons(c[0], c[1])
                    .iter()
                    .flat_map(|r| [r.left, r.top, r.right, r.bottom])
                    .collect::<Vec<_>>(),
            )
        })
    }
}
