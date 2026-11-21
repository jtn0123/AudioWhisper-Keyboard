//! Offscreen Vello CPU drawing for small chrome views and settings images.
use crate::{
    handle,
    jni::{self, illegal},
    render::{Session, strings},
};
use jni_sys::{JNIEnv, jfloatArray, jintArray, jlong, jobject, jobjectArray};
use std::ptr;
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_chrome(
    env: *mut JNIEnv,
    _: jobject,
    native: jlong,
    config: jfloatArray,
    labels: jobjectArray,
    icons: jobjectArray,
    metadata: jobjectArray,
) {
    // SAFETY: synchronized JVM owner retains the session; inputs are copied before drawing.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let c = env.read::<f32>(config, "chrome config")?;
            let metadata = strings(env, metadata)?;
            if metadata.len() != 2 {
                return Err(illegal("chrome metadata"));
            }
            s.mode = metadata[0].clone();
            s.shift = metadata[1].clone();
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
                    ..Default::default()
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
    _: jobject,
    native: jlong,
) -> jintArray {
    // SAFETY: synchronized JVM owner retains the session; returned pixels are a fresh primitive array.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            s.prepare_panel();
            let raster = s.bitmap.get_or_insert_with(|| {
                render::raster::Raster::shared(s.scene.width, s.scene.height, s.fonts.clone())
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
            let pixels = argb(raster.pixels());
            env.make(&pixels)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ImeChromeRules_oneHanded(
    env: *mut JNIEnv,
    _: jobject,
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
    _: jobject,
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

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_chromeInfo(
    env: *mut JNIEnv,
    _: jobject,
    native: jlong,
) -> jfloatArray {
    // SAFETY: synchronized JVM owner retains the session; output values are copied.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            let c = s
                .scene
                .panel
                .as_ref()
                .and_then(|p| p.chrome.as_ref())
                .ok_or_else(|| illegal("chrome missing"))?;
            env.make(&c.field)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ImeChromeRules_extract(
    env: *mut JNIEnv,
    _: jobject,
    config: jfloatArray,
) -> jfloatArray {
    // SAFETY: local input is copied; output is a new primitive array.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<f32>(config, "extract config")?;
            if c.len() != 5 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
                return Err(illegal("extract config"));
            }
            env.make(
                &render::extract_chrome::geometry(c[0], c[1], c[2], [c[3], c[4]])
                    .iter()
                    .flat_map(|r| [r.left, r.top, r.right, r.bottom])
                    .collect::<Vec<_>>(),
            )
        })
    }
}

pub(crate) fn argb(pixels: &[u8]) -> Vec<i32> {
    pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            let a = p[3] as u32;
            let channel = |v: u8| {
                (v as u32 * 255 + a / 2)
                    .checked_div(a)
                    .map_or(0, |n| n.min(255))
            };
            ((a << 24) | (channel(p[0]) << 16) | (channel(p[1]) << 8) | channel(p[2])) as i32
        })
        .collect()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ImeChromeRules_previewSize(
    env: *mut JNIEnv,
    _: jobject,
    config: jfloatArray,
) -> jintArray {
    // SAFETY: local input is copied; output is a new JVM-owned primitive array.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<f32>(config, "preview config")?;
            if c.len() != 2 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
                return Err(illegal("preview config"));
            }
            env.make(&render::key_preview::size(c[0], c[1]).map(i32::from))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ImeChromeRules_startEllipsis(
    env: *mut JNIEnv,
    _: jobject,
    labels: jobjectArray,
    config: jfloatArray,
) -> jobjectArray {
    // SAFETY: inputs and output strings are local JVM values copied across the boundary.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let labels = strings(env, labels)?;
            let c = env.read::<f32>(config, "ellipsis config")?;
            if c.len() != 2 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
                return Err(illegal("ellipsis config"));
            }
            let output: Vec<Vec<u16>> = labels
                .iter()
                .map(|label| {
                    render::ime_decorations::start_ellipsis(label, c[0], |s| {
                        s.encode_utf16().count() as f32 * c[1]
                    })
                    .encode_utf16()
                    .collect()
                })
                .collect();
            env.make_strings(&output)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ImeChromeRules_minimumTarget(
    env: *mut JNIEnv,
    _: jobject,
    config: jfloatArray,
) -> jfloatArray {
    // SAFETY: the JVM input is copied; the result is a fresh local primitive array.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<f32>(config, "minimum target")?;
            if c.len() != 6 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
                return Err(illegal("minimum target"));
            }
            let header = c[5] == 0.0;
            let mut item = render::panels::Item::new(
                1,
                render::panels::rect(0.0, c[2], c[1], c[3]),
                "label".into(),
                if header {
                    "subtype-panel-header"
                } else {
                    "extracted-landscape-input-action"
                },
                render::panels::Command::None,
            );
            item.clip = render::panels::rect(0.0, 0.0, c[1], 10000.0);
            let mut items = vec![item];
            if header {
                items.push(render::panels::Item::new(
                    2,
                    render::panels::rect(0.0, c[4], c[1], 56.0 * c[0]),
                    "row".into(),
                    "subtype-panel-list-item",
                    render::panels::Command::Subtype(Some(0)),
                ));
            }
            let panel = render::panels::Panel {
                density: c[0],
                items,
                ..Default::default()
            };
            let r = panel.target_bounds(0);
            env.make(&[r.left, r.top, r.right, r.bottom])
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ImeChromeRules_fontMultiplier(
    env: *mut JNIEnv,
    _: jobject,
    config: jintArray,
) -> jni_sys::jfloat {
    // SAFETY: the local primitive array is copied before native policy is called.
    unsafe {
        jni::run(env, 0.0, |env| {
            let c = env.read::<i32>(config, "font multiplier")?;
            if c.len() != 4 {
                return Err(illegal("font multiplier"));
            }
            Ok(render::ime_chrome::font_multiplier(
                c[0],
                c[1],
                c[2] != 0,
                c[3] != 0,
            ))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ImeChromeRules_rowHeights(
    env: *mut JNIEnv,
    _: jobject,
    base: jni_sys::jfloat,
    config: jintArray,
) -> jfloatArray {
    // SAFETY: copied local inputs produce a fresh JVM-owned primitive array.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<i32>(config, "row heights")?;
            if c.len() != 5 || !base.is_finite() || base < 0.0 {
                return Err(illegal("row heights"));
            }
            env.make(&render::ime_chrome::row_heights(
                base,
                c[0],
                c[1],
                c[2],
                c[3] != 0,
                c[4] != 0,
            ))
        })
    }
}
