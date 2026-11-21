//! Thin JNI boundary for the prototype. All Java arrays are copied before scene work.
use crate::{
    handle,
    jni::{self, Result, illegal},
};
use jni_sys::{
    JNIEnv, jboolean, jfloatArray, jint, jintArray, jlong, jobject, jobjectArray, jstring,
};
use keys::{Bounds, Rect};
use render::{
    scene::{Key, Popup, Scene},
    touch::Touch,
};
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct Session {
    pub(crate) scene: Scene,
    pub(crate) settings: Option<render::settings_ui::App>,
    pub(crate) text: Option<render::text::Text>,
    pub(crate) touch: Touch,
    pub(crate) semantics: render::accessibility::Semantics,
    pub(crate) fonts: Vec<render::text::FontBlob>,
    pub(crate) bitmap: Option<render::raster::Raster>,
    pub(crate) touch_time: i64,
    pub(crate) theme: Option<render::theme::Theme>,
    pub(crate) mode: String,
    pub(crate) shift: String,
    types: Vec<String>,
    popup_codes: Vec<i32>,
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
    _: jobject,
    paths: jobjectArray,
) -> jlong {
    // SAFETY: the JVM supplies this call's environment and array references.
    unsafe {
        jni::run(env, 0, |env| {
            #[cfg(target_os = "android")]
            crate::render_profile::heap(0, 0, 0);
            let mut fonts = Vec::new();
            for path in env.read_strings(paths, "fonts")? {
                let path = String::from_utf16(&path.ok_or_else(|| illegal("null font path"))?)
                    .map_err(|_| illegal("invalid font path"))?;
                fonts.push(render::text::FontBlob::from(
                    std::fs::read(path).map_err(|_| illegal("could not read system font"))?,
                ));
            }
            #[cfg(target_os = "android")]
            crate::render_profile::heap(1, fonts.iter().map(|f| f.len()).sum(), 0);
            Ok(handle::into_handle(Mutex::new(Session {
                text: Some(render::text::Text::shared(fonts.clone())),
                fonts,
                ..Session::default()
            })))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_theme(
    env: *mut JNIEnv,
    _: jobject,
    native: jlong,
    json: jstring,
    names: jobjectArray,
    colors: jintArray,
    scale: jfloatArray,
) {
    // SAFETY: the synchronized wrapper owns the live handle and all local JVM references.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let json = String::from_utf16(&env.read_string(json, "stylesheet")?)
                .map_err(|_| illegal("invalid stylesheet UTF-16"))?;
            let names = strings(env, names)?;
            let colors = env.read::<i32>(colors, "palette")?;
            let scale = env.read::<f32>(scale, "scale")?;
            s.theme = Some(crate::render_theme::theme(&json, &names, &colors, &scale)?);
            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_update(
    env: *mut JNIEnv,
    _: jobject,
    native: jlong,
    dimensions: jfloatArray,
    metadata: jobjectArray,
    candidate_flags: jintArray,
    geometry: jfloatArray,
    labels: jobjectArray,
    candidates: jobjectArray,
) {
    // SAFETY: Kotlin synchronizes access and keeps the handle alive until close.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |session, env| {
            let d = env.read::<f32>(dimensions, "dimensions")?;
            let meta = strings(env, metadata)?;
            let flags = env.read::<i32>(candidate_flags, "candidate flags")?;
            let g = env.read::<f32>(geometry, "geometry")?;
            let labels = strings(env, labels)?;
            let candidates = strings(env, candidates)?;
            if ![3, 5, 6].contains(&d.len())
                || d.iter().any(|v| !v.is_finite())
                || !(1.0..=8192.0).contains(&d[0])
                || !(1.0..=8192.0).contains(&d[1])
                || d[2] < 0.0
                || d[2] >= d[1]
                || g.len() % 10 != 0
                || g.iter().any(|v| !v.is_finite())
            {
                return Err(illegal("invalid scene dimensions"));
            }
            let count = g.len() / 10;
            if labels.len() != count * 3
                || meta.len() != 2 + count
                || flags.len() != candidates.len()
            {
                return Err(illegal("invalid key scene arrays"));
            }
            let keys = g
                .as_chunks::<10>()
                .0
                .iter()
                .enumerate()
                .map(|(i, g)| Key {
                    code: g[9] as i32,
                    bounds: Bounds {
                        touch: rectangle(g),
                        visible: rectangle(&g[4..]),
                    },
                    label: labels[i * 3].clone(),
                    hint: labels[i * 3 + 1].clone(),
                    icon: labels[i * 3 + 2].clone(),
                    enabled: g[8] != 0.0,
                    styles: Default::default(),
                    hints: Default::default(),
                })
                .collect();
            if d.len() >= 5 {
                session.scene.desired_key = [d[3], d[4]];
            }
            if session.scene.width != d[0] as u16 || session.scene.height != d[1] as u16 {
                session.bitmap = None;
            }
            session.scene.keyboard_height = if d.len() == 6 { d[5] } else { d[1] - d[2] };
            session.scene.width = d[0] as u16;
            session.scene.height = d[1] as u16;
            session.scene.strip = d[2];
            session.scene.keys = keys;
            session.scene.candidates = candidates;
            session.mode = meta[0].clone();
            session.shift = meta[1].clone();
            session.types = meta[2..].to_vec();
            let theme = session
                .theme
                .as_mut()
                .ok_or_else(|| illegal("renderer theme not set"))?;
            theme.apply(
                &mut session.scene,
                &session.mode,
                &session.shift,
                &session.types,
                &flags,
            );
            render::smartbar_scene::Smartbar::prepare(
                &mut session.scene,
                theme,
                &render::theme::Request {
                    element: String::new(),
                    mode: session.mode.clone(),
                    shift: session.shift.clone(),
                    code: 0,
                    key_type: String::new(),
                    selector: 0,
                    auto_commit: false,
                    popup_source: None,
                },
                session
                    .text
                    .as_mut()
                    .ok_or_else(|| illegal("missing system fonts"))?,
                &flags,
            );
            if let Some(popup) = &session.scene.popup {
                let key = popup.key;
                if let Some(k) = session.scene.keys.get(key) {
                    let request = render::theme::Request {
                        element: String::new(),
                        mode: session.mode.clone(),
                        shift: session.shift.clone(),
                        code: k.code,
                        key_type: session.types[key].clone(),
                        selector: 0,
                        auto_commit: false,
                        popup_source: None,
                    };
                    theme.popup_styles(&mut session.scene, &request, &session.popup_codes);
                }
            }
            Ok(())
        })
    }
}

pub(crate) fn strings(env: &jni::Env, array: jobjectArray) -> Result<Vec<String>> {
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
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_popup(
    env: *mut JNIEnv,
    _: jobject,
    native: jlong,
    key: jint,
    labels: jobjectArray,
    codes: jintArray,
) {
    // SAFETY: synchronized Kotlin wrapper keeps the handle alive.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let labels = strings(env, labels)?;
            let codes = env.read::<i32>(codes, "popup codes")?;
            if codes.len() != labels.len() {
                return Err(illegal("popup code count"));
            }
            if key < 0 || key as usize >= s.scene.keys.len() || labels.is_empty() {
                return Err(illegal("invalid popup"));
            }
            let key_index = key as usize;
            let request = render::theme::Request {
                element: String::new(),
                mode: s.mode.clone(),
                shift: s.shift.clone(),
                code: s.scene.keys[key_index].code,
                key_type: s.types[key_index].clone(),
                selector: 0,
                auto_commit: false,
                popup_source: None,
            };
            s.theme
                .as_mut()
                .ok_or_else(|| illegal("renderer theme not set"))?
                .popup_styles(&mut s.scene, &request, &codes);
            s.popup_codes = codes;
            s.scene.popup_geometry = None;
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
    _: jobject,
    native: jlong,
    surface: jobject,
    hybrid: jboolean,
    timeline: jboolean,
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
                    timeline != 0,
                )?);
            }
            #[cfg(not(target_os = "android"))]
            {
                let _ = (s.fonts.len(), surface, hybrid, timeline);
            }
            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_frame(
    env: *mut JNIEnv,
    _: jobject,
    native: jlong,
    time: jlong,
    vsync: jlong,
) -> jboolean {
    // SAFETY: synchronized wrapper owns the handle during the call.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", 0, |s, _| {
            s.prepare_panel();
            let mut animating = s.touch.animate(&mut s.scene, time);
            if let Some(g) = s.scene.panel.as_mut().and_then(|p| p.gif.as_mut())
                && (g.status == 0 || g.sending.is_some())
            {
                g.phase = (time % 1_332_000_000) as f32 / 1_332_000_000.0;
                animating = true;
            }
            if let Some(bar) = &mut s.scene.smartbar
                && bar.actions.iter().any(|a| a.warming)
            {
                bar.phase = (time % 900_000_000) as f32 / 900_000_000.0;
            }
            #[cfg(target_os = "android")]
            if let Some(worker) = &s.worker {
                worker.submit(s.scene.clone(), s.touch_time, time, vsync);
            }
            #[cfg(not(target_os = "android"))]
            {
                let _ = (s, time, vsync);
            }
            Ok(u8::from(animating))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_detach(
    env: *mut JNIEnv,
    _: jobject,
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
    _: jobject,
    native: jlong,
) {
    // SAFETY: the synchronized wrapper frees its handle once, then sets it to zero.
    unsafe {
        handle::free::<Mutex<Session>>(native);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_popupGrid(
    env: *mut JNIEnv,
    _: jobject,
    native: jlong,
    prioritized: jint,
    landscape: jboolean,
) {
    // SAFETY: synchronized owner retains the native session and its prepared popup through this call.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, _| {
            let order = render::popup_scene::configure(&mut s.scene, prioritized, landscape != 0)
                .ok_or_else(|| illegal("invalid popup grid"))?;
            s.popup_codes = order.iter().map(|&i| s.popup_codes[i]).collect();
            Ok(())
        });
    }
}
