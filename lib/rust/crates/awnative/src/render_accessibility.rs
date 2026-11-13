//! JNI transport for the Rust semantic scene. Local strings are never logged.
use crate::{
    handle,
    jni::{self, illegal},
    render::{Session, strings},
};
use jni_sys::{JNIEnv, jclass, jint, jintArray, jlong, jobjectArray, jstring};
use render::accessibility_labels::Labels;
use serde_json::json;
use std::ptr;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_semanticCancel(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
) -> jintArray {
    // SAFETY: synchronized owner retains the session and receives a new local action array.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            let actions: Vec<_> = s
                .semantics
                .dismiss(&mut s.scene)
                .into_iter()
                .flat_map(|a| [a.0, a.1, a.2])
                .collect();
            s.prepare_panel();
            env.make(&actions)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_semanticConfig(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    names: jobjectArray,
    values: jobjectArray,
    text: jobjectArray,
    popups: jintArray,
    removable: jintArray,
    checked: jintArray,
    editor: jintArray,
) {
    // SAFETY: synchronized JVM owner retains the session and all local arrays for this call.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let names = strings(env, names)?;
            let values = strings(env, values)?;
            if names.len() != values.len() {
                return Err(illegal("semantic resource count"));
            }
            s.semantics.labels = Labels(names.into_iter().zip(values).collect());
            s.semantics.action_text = strings(env, text)?;
            s.semantics.popup_keys = env
                .read::<i32>(popups, "popup availability")?
                .iter()
                .map(|v| *v != 0)
                .collect();
            s.semantics.removable = env
                .read::<i32>(removable, "candidate actions")?
                .iter()
                .map(|v| *v != 0)
                .collect();
            s.semantics.checked_codes = env.read(checked, "checked codes")?;
            s.semantics.editor = env
                .read::<i32>(editor, "editor options")?
                .try_into()
                .map_err(|_| illegal("editor option count"))?;
            Ok(())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_semanticSnapshot(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
) -> jstring {
    // SAFETY: synchronized JVM owner retains the session; the new local string belongs to JVM.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            let nodes: Vec<_> = s.semantics.snapshot(&s.scene, &s.shift).into_iter().map(|n| json!({
                "id": n.id, "bounds": [n.bounds.left, n.bounds.top, n.bounds.right, n.bounds.bottom],
                "label": n.label, "description": n.description, "role": n.role, "state": n.state,
                "enabled": n.enabled, "pressed": n.pressed, "selected": n.selected,
                "checked": n.checked, "locked": n.locked, "actions": n.actions, "longClickLabel": n.long_click_label,
                "holdDelayMillis": n.hold_delay(&s.scene, s.touch.long_press_delay / 1_000_000),
            })).collect();
            let value = json!({"nodes": nodes, "mode": s.semantics.labels.mode(&s.mode),
                "shift": s.semantics.labels.shift(&s.shift), "popup": s.scene.popup.is_some() || s.scene.panel.as_ref().and_then(|p|p.emoji.as_ref()).is_some_and(|e|e.popup.is_some())});
            env.make_string(&value.to_string().encode_utf16().collect::<Vec<_>>())
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_semanticAction(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    id: jint,
    action: jint,
) -> jintArray {
    // SAFETY: synchronized owner retains the session; copied action packets belong to JVM.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            let actions: Vec<_> = s
                .semantics
                .activate(&mut s.scene, &s.shift, id, action)
                .into_iter()
                .flat_map(|a| [a.0, a.1, a.2])
                .collect();
            s.prepare_panel();
            env.make(&actions)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SemanticRules_labels(
    env: *mut JNIEnv,
    _: jclass,
    codes: jintArray,
    labels: jobjectArray,
    shift: jstring,
    bar: jint,
) -> jobjectArray {
    // SAFETY: JVM local arrays/strings are copied during this invocation; output is a local array.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let codes = env.read::<i32>(codes, "codes")?;
            let labels = strings(env, labels)?;
            let shift = String::from_utf16_lossy(&env.read_string(shift, "shift")?);
            if codes.len() != labels.len() {
                return Err(illegal("semantic label count"));
            }
            let rules = Labels::default();
            let result: Vec<Vec<u16>> = codes
                .iter()
                .zip(labels)
                .map(|(&code, label)| {
                    (if bar < 0 {
                        rules.key(code, &label, &shift)
                    } else {
                        rules.action(code, &label, bar)
                    })
                    .encode_utf16()
                    .collect()
                })
                .collect();
            env.make_strings(&result)
        })
    }
}
