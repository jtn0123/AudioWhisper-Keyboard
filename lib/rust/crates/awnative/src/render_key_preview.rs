//! A settings key preview uses the same theme resolution and CPU raster as the keyboard.
use crate::{
    handle,
    jni::illegal,
    render::{Session, strings},
};
use jni_sys::{JNIEnv, jclass, jint, jintArray, jlong, jobjectArray};
use keys::{Bounds, Rect};
use render::{
    scene::{Key, Scene},
    theme::Request,
};
use std::ptr;
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_keyPreview(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    code: jint,
    labels: jobjectArray,
) -> jintArray {
    // SAFETY: synchronized JVM owner keeps the session live; local strings/pixels are copied.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            let labels = strings(env, labels)?;
            if labels.len() != 3 {
                return Err(illegal("key preview labels"));
            }
            let theme = s
                .theme
                .as_mut()
                .ok_or_else(|| illegal("preview theme missing"))?;
            let text = s
                .text
                .as_mut()
                .ok_or_else(|| illegal("preview fonts missing"))?;
            let request = Request {
                element: "key".into(),
                mode: "numeric_advanced".into(),
                shift: "unshifted".into(),
                code,
                key_type: labels[0].clone(),
                selector: 0,
                auto_commit: false,
                popup_source: None,
            };
            let style = theme.style(&request);
            let label = if labels[2].is_empty() {
                labels[1].as_str()
            } else {
                ""
            };
            let [width, height] = render::key_preview::size(
                theme.density(),
                text.measure(label, style.size, style.mono, style.weight),
            );
            let bounds = Rect {
                left: 0.0,
                top: 0.0,
                right: width as f32,
                bottom: height as f32,
            };
            let scene = Scene {
                width,
                height,
                keys: vec![Key {
                    code,
                    bounds: Bounds {
                        touch: bounds,
                        visible: bounds,
                    },
                    label: label.into(),
                    hint: String::new(),
                    icon: labels[2].clone(),
                    styles: [style.clone(), style],
                    hints: Default::default(),
                    enabled: true,
                }],
                ..Default::default()
            };
            let mut raster = render::raster::Raster::new(width, height, s.fonts.clone());
            raster
                .draw(&scene, bounds)
                .map_err(|_| illegal("key preview raster failed"))?;
            let mut out = vec![width as i32, height as i32];
            out.extend(crate::render_chrome::argb(raster.pixels()));
            env.make(&out)
        })
    }
}
