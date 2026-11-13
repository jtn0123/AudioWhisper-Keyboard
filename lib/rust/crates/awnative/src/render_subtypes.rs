//! Subtype resource transport; order, selection, layout and input are native.
use crate::{
    handle,
    jni::illegal,
    render::{Session, strings},
};
use jni_sys::{JNIEnv, jclass, jlong, jlongArray, jobjectArray};
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_subtypes(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    selected: jlong,
    ids: jlongArray,
    labels: jobjectArray,
) {
    // SAFETY: synchronized JVM owner retains the session; local arrays are copied.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let ids = env.read::<i64>(ids, "subtype ids")?;
            let labels = strings(env, labels)?;
            if labels.len() != ids.len() + 3 {
                return Err(illegal("subtype labels"));
            }
            let p = s
                .scene
                .panel
                .as_mut()
                .ok_or_else(|| illegal("subtype panel missing"))?;
            if p.kind != 7 {
                return Err(illegal("wrong subtype panel"));
            }
            p.subtypes = Some(render::subtype_panel::Subtypes {
                ids,
                selected,
                header: labels[0].clone(),
                icons: [labels[1].clone(), labels[2].clone()],
                labels: labels[3..].to_vec(),
                ..Default::default()
            });
            s.prepare_panel();
            Ok(())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SubtypeRules_rows(
    env: *mut JNIEnv,
    _: jclass,
    selected: jlong,
    ids: jlongArray,
    labels: jobjectArray,
) -> jobjectArray {
    // SAFETY: local arrays are copied; the output is a fresh JVM-owned string array.
    unsafe {
        crate::jni::run(env, std::ptr::null_mut(), |env| {
            let ids = env.read::<i64>(ids, "ids")?;
            let labels = strings(env, labels)?;
            if ids.len() != labels.len() {
                return Err(illegal("subtype count"));
            }
            let out: Vec<Vec<u16>> = render::subtype_panel::rows(&ids, &labels, selected)
                .into_iter()
                .flat_map(|(id, label, selected)| [id.to_string(), label, selected.to_string()])
                .map(|s| s.encode_utf16().collect())
                .collect();
            env.make_strings(&out)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SubtypeRules_geometry(
    env: *mut JNIEnv,
    _: jclass,
    config: jni_sys::jfloatArray,
) -> jni_sys::jfloatArray {
    // SAFETY: the JVM local input is copied; the result is a fresh primitive array.
    unsafe {
        crate::jni::run(env, std::ptr::null_mut(), |env| {
            let c = env.read::<f32>(config, "subtype geometry")?;
            if c.len() != 6 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
                return Err(illegal("subtype geometry"));
            }
            env.make(&[
                render::subtype_panel::sheet_start(c[0], c[1], c[2], c[3] as usize),
                render::modal_scene::offset(c[0], c[4], c[5]),
            ])
        })
    }
}
