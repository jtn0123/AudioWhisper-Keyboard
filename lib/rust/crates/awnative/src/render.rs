//! The renderer's session, for `RenderNative`, whose generated entry points call the functions
//! here and in the other render_* modules. Kotlin's `KeyboardRenderer` holds it as a handle and
//! calls under its own lock; each call runs on the session through [`session`].
use crate::{
    handle,
    jni::{Env, Result, illegal},
};
use jni_sys::jobject;
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

/// A new renderer with the fonts at `paths`.
pub fn create(paths: &[Vec<u16>]) -> Result<i64> {
    #[cfg(target_os = "android")]
    crate::render_profile::heap(0, 0, 0);
    let mut fonts = Vec::new();
    for path in paths {
        let path = String::from_utf16(path).map_err(|_| illegal("invalid font path"))?;
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
}

/// Runs `body` on the renderer behind `native`, locked; a closed renderer throws.
pub(crate) fn session<R>(native: i64, body: impl FnOnce(&mut Session) -> Result<R>) -> Result<R> {
    // SAFETY: the synchronized Kotlin wrapper holds the renderer while it calls, and frees it
    // only on close, under the same lock, leaving 0.
    body(&mut *unsafe { handle::locked::<Session>(native, "renderer")? })
}

pub fn theme(
    native: i64,
    json: &[u16],
    names: &[Vec<u16>],
    colors: &[i32],
    scale: &[f32],
) -> Result<()> {
    session(native, |s| {
        let json = String::from_utf16(json).map_err(|_| illegal("invalid stylesheet UTF-16"))?;
        let names = texts(names);
        s.theme = Some(crate::render_theme::theme(&json, &names, colors, scale)?);
        Ok(())
    })
}

pub fn update(
    native: i64,
    dimensions: &[f32],
    metadata: &[Vec<u16>],
    candidate_flags: &[i32],
    geometry: &[f32],
    labels: &[Vec<u16>],
    candidates: &[Vec<u16>],
) -> Result<()> {
    session(native, |session| {
        let d = dimensions;
        let meta = texts(metadata);
        let flags = candidate_flags;
        let g = geometry;
        let labels = texts(labels);
        let candidates = texts(candidates);
        if ![3, 5, 6].contains(&d.len())
            || d.iter().any(|v| !v.is_finite())
            || !(1.0..=8192.0).contains(&d[0])
            || !(1.0..=8192.0).contains(&d[1])
            || d[2] < 0.0
            || d[2] >= d[1]
            || !g.len().is_multiple_of(10)
            || g.iter().any(|v| !v.is_finite())
        {
            return Err(illegal("invalid scene dimensions"));
        }
        let count = g.len() / 10;
        if labels.len() != count * 3 || meta.len() != 2 + count || flags.len() != candidates.len() {
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
            flags,
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
            flags,
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

/// `texts` as Rust strings, invalid UTF-16 shown as replacement glyphs, as `strings` reads them.
pub(crate) fn texts(texts: &[Vec<u16>]) -> Vec<String> {
    texts
        .iter()
        .map(|text| String::from_utf16_lossy(text))
        .collect()
}

/// A string array as Rust strings, as [`texts`], for the JVM test probes, which read their own.
#[cfg(not(target_os = "android"))]
pub(crate) fn strings(env: &Env, array: jni_sys::jobjectArray) -> Result<Vec<String>> {
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

pub fn popup(native: i64, key: i32, labels: &[Vec<u16>], codes: &[i32]) -> Result<()> {
    session(native, |s| {
        let labels = texts(labels);
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
            .popup_styles(&mut s.scene, &request, codes);
        s.popup_codes = codes.to_vec();
        s.scene.popup_geometry = None;
        s.scene.popup = Some(Popup {
            key: key as usize,
            labels,
            selected: 0,
        });
        Ok(())
    })
}

/// Draws to `surface`, an Android `Surface`, from a worker thread, which owns the native window
/// it acquires.
pub fn attach(
    env: &Env,
    native: i64,
    surface: jobject,
    hybrid: bool,
    timeline: bool,
) -> Result<()> {
    session(native, |s| {
        #[cfg(target_os = "android")]
        {
            s.worker.take();
            s.worker = Some(crate::render_window::Worker::new(
                env.raw(),
                surface,
                s.fonts.clone(),
                s.scene.width,
                s.scene.height,
                hybrid,
                timeline,
            )?);
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = (env, s.fonts.len(), surface, hybrid, timeline);
        }
        Ok(())
    })
}

pub fn frame(native: i64, time: i64, vsync: i64) -> Result<bool> {
    session(native, |s| {
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
        Ok(animating)
    })
}

pub fn detach(native: i64) -> Result<()> {
    session(native, |s| {
        #[cfg(target_os = "android")]
        s.worker.take();
        s.touch.cancel(&mut s.scene);
        Ok(())
    })
}

pub fn close(native: i64) -> Result<()> {
    // SAFETY: the synchronized wrapper frees its handle once, then sets it to zero.
    unsafe { handle::free::<Mutex<Session>>(native) };
    Ok(())
}

pub fn popup_grid(native: i64, prioritized: i32, landscape: bool) -> Result<()> {
    session(native, |s| {
        let order = render::popup_scene::configure(&mut s.scene, prioritized, landscape)
            .ok_or_else(|| illegal("invalid popup grid"))?;
        s.popup_codes = order.iter().map(|&i| s.popup_codes[i]).collect();
        Ok(())
    })
}
