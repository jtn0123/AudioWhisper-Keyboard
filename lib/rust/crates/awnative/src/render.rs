//! Thin JNI boundary for the prototype. All Java arrays are copied before scene work.
use crate::{
    handle,
    jni::{self, Result, illegal},
};
use jni_sys::{
    JNIEnv, jboolean, jclass, jfloatArray, jint, jintArray, jlong, jlongArray, jobject,
    jobjectArray,
};
use keys::{Bounds, Rect};
use render::{
    scene::{Key, Popup, Scene},
    touch::{Motion, Touch},
};
use std::{ptr, sync::Mutex};

#[derive(Debug, Default)]
pub struct Session {
    scene: Scene,
    touch: Touch,
    fonts: Vec<Vec<u8>>,
    touch_time: i64,
    #[cfg(target_os = "android")]
    worker: Option<crate::render_window::Worker>,
}

fn rectangle(a: &[f32]) -> Rect {
    Rect {
        left: a[0],
        top: a[1],
        right: a[2],
        bottom: a[3],
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_create(
    env: *mut JNIEnv,
    _: jclass,
    paths: jobjectArray,
) -> jlong {
    // SAFETY: the JVM supplies this call's environment and array references.
    unsafe {
        jni::run(env, 0, |env| {
            let mut fonts = Vec::new();
            for path in env.read_strings(paths, "fonts")? {
                let path = String::from_utf16(&path.ok_or_else(|| illegal("null font path"))?)
                    .map_err(|_| illegal("invalid font path"))?;
                fonts.push(std::fs::read(path).map_err(|_| illegal("could not read system font"))?);
            }
            Ok(handle::into_handle(Mutex::new(Session {
                fonts,
                ..Session::default()
            })))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_update(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    dimensions: jfloatArray,
    colors: jintArray,
    geometry: jfloatArray,
    labels: jobjectArray,
    candidates: jobjectArray,
) {
    // SAFETY: Kotlin synchronizes access and keeps the handle alive until close.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |session, env| {
            let d = env.read::<f32>(dimensions, "dimensions")?;
            let colors = env.read::<i32>(colors, "colors")?;
            let g = env.read::<f32>(geometry, "geometry")?;
            let labels = strings(env, labels)?;
            let candidates = strings(env, candidates)?;
            if d.len() != 5
                || d.iter().any(|v| !v.is_finite())
                || !(1.0..=8192.0).contains(&d[0])
                || !(1.0..=8192.0).contains(&d[1])
                || d[2] < 0.0
                || d[2] >= d[1]
                || colors.len() < 5
                || g.len() % 14 != 0
                || g.iter().any(|v| !v.is_finite())
            {
                return Err(illegal("invalid scene dimensions"));
            }
            let count = g.len() / 14;
            if labels.len() != count * 3 || colors.len() != 5 + count * 6 {
                return Err(illegal("invalid key scene arrays"));
            }
            let keys = g
                .as_chunks::<14>()
                .0
                .iter()
                .enumerate()
                .map(|(i, g)| Key {
                    code: g[13] as i32,
                    bounds: Bounds {
                        touch: rectangle(g),
                        visible: rectangle(&g[4..]),
                    },
                    label: labels[i * 3].clone(),
                    hint: labels[i * 3 + 1].clone(),
                    icon: labels[i * 3 + 2].clone(),
                    size: g[8],
                    radius: g[9],
                    enabled: g[10] != 0.0,
                    hint_size: g[11],
                    shadow: g[12].max(0.0),
                    hint_colors: [colors[5 + i * 6 + 4] as u32, colors[5 + i * 6 + 5] as u32],
                    colors: std::array::from_fn(|c| colors[5 + i * 6 + c] as u32),
                })
                .collect();
            session.scene.width = d[0] as u16;
            session.scene.height = d[1] as u16;
            session.scene.strip = d[2];
            session.scene.candidate_size = d[3];
            session.scene.background = colors[0] as u32;
            session.scene.candidate_color = colors[1] as u32;
            session.scene.popup_colors = [colors[2] as u32, colors[3] as u32, colors[4] as u32];
            session.scene.keys = keys;
            session.scene.candidates = candidates;
            Ok(())
        })
    }
}

fn strings(env: &jni::Env, array: jobjectArray) -> Result<Vec<String>> {
    env.read_strings(array, "strings")?
        .into_iter()
        .map(|s| {
            // Display invalid UTF-16 as replacement glyphs; the Kotlin KeyData used for
            // committing text is untouched, as it is for Compose's text rendering.
            Ok(String::from_utf16_lossy(
                &s.ok_or_else(|| illegal("null scene string"))?,
            ))
        })
        .collect()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_touch(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    metadata: jintArray,
    times: jlongArray,
    coords: jfloatArray,
) -> jintArray {
    // SAFETY: the wrapper owns the live handle and JVM arrays for this call.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            let meta = env.read::<i32>(metadata, "metadata")?;
            if meta.len() < 2 {
                return Err(illegal("missing action/index"));
            }
            let event = Motion {
                action: meta[0],
                index: meta[1] as usize,
                ids: meta[2..].to_vec(),
                times: env.read(times, "times")?,
                coords: env.read(coords, "coords")?,
            };
            s.touch_time = event.times.last().copied().unwrap_or(0);
            let actions: Vec<i32> = s
                .touch
                .event(&mut s.scene, &event)
                .into_iter()
                .flat_map(|a| [a.0, a.1, a.2])
                .collect();
            env.make(&actions)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_popup(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    key: jint,
    labels: jobjectArray,
) {
    // SAFETY: synchronized Kotlin wrapper keeps the handle alive.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let labels = strings(env, labels)?;
            if key < 0 || key as usize >= s.scene.keys.len() || labels.is_empty() {
                return Err(illegal("invalid popup"));
            }
            s.scene.popup = Some(Popup {
                key: key as usize,
                labels,
                selected: 0,
            });
            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_attach(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    surface: jobject,
    hybrid: jboolean,
) {
    // SAFETY: live session and local Surface reference are valid for this call. Worker owns the acquired native reference.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, _| {
            #[cfg(target_os = "android")]
            {
                s.worker.take();
                s.worker = Some(crate::render_window::Worker::new(
                    env,
                    surface,
                    s.fonts.clone(),
                    s.scene.width,
                    s.scene.height,
                    hybrid != 0,
                )?);
            }
            #[cfg(not(target_os = "android"))]
            {
                let _ = (s.fonts.len(), surface, hybrid);
            }
            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_frame(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    time: jlong,
) {
    // SAFETY: synchronized wrapper owns the handle during the call.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, _| {
            #[cfg(target_os = "android")]
            if let Some(worker) = &s.worker {
                worker.submit(s.scene.clone(), s.touch_time, time);
            }
            #[cfg(not(target_os = "android"))]
            {
                let _ = (s, time);
            }
            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_detach(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
) {
    // SAFETY: synchronized wrapper owns the handle. Dropping joins the thread before releasing the native window.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, _| {
            #[cfg(target_os = "android")]
            s.worker.take();
            s.touch.cancel(&mut s.scene);
            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_close(
    _: *mut JNIEnv,
    _: jclass,
    native: jlong,
) {
    // SAFETY: the synchronized wrapper frees its handle once, then sets it to zero.
    unsafe {
        handle::free::<Mutex<Session>>(native);
    }
}
