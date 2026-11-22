//! JNI transport for the Rust semantic scene. Local strings are never logged.
use crate::{
    boundary::args::RenderNativeSemanticConfig,
    jni::{Result, illegal},
    render::{session, texts},
};
use render::accessibility_labels::Labels;
use serde_json::json;

pub fn semantic_cancel(native: i64) -> Result<Vec<i32>> {
    session(native, |s| {
        let actions: Vec<_> = s
            .semantics
            .dismiss(&mut s.scene)
            .into_iter()
            .flat_map(|a| [a.0, a.1, a.2])
            .collect();
        s.prepare_panel();
        Ok(actions)
    })
}

pub fn semantic_config(args: RenderNativeSemanticConfig) -> Result<()> {
    let RenderNativeSemanticConfig {
        handle,
        names,
        values,
        text,
        popups,
        removable,
        checked,
        editor,
    } = args;
    session(handle, |s| {
        let names = texts(names);
        let values = texts(values);
        if names.len() != values.len() {
            return Err(illegal("semantic resource count"));
        }
        s.semantics.labels = Labels(names.into_iter().zip(values).collect());
        s.semantics.action_text = texts(text);
        s.semantics.popup_keys = popups.iter().map(|v| *v != 0).collect();
        // A held bubble's dots follow the key's choices, which arrive after the keys.
        s.style_bubble(true);
        s.semantics.removable = removable.iter().map(|v| *v != 0).collect();
        s.semantics.checked_codes = checked.to_vec();
        s.semantics.editor = editor
            .try_into()
            .map_err(|_| illegal("editor option count"))?;
        Ok(())
    })
}
pub fn semantic_snapshot(native: i64) -> Result<Vec<u16>> {
    session(native, |s| {
        let nodes: Vec<_> = s.semantics.snapshot(&s.scene, &s.shift).into_iter().map(|n| json!({
            "id": n.id, "bounds": [n.bounds.left, n.bounds.top, n.bounds.right, n.bounds.bottom],
            "label": n.label, "description": n.description, "role": n.role, "state": n.state,
            "enabled": n.enabled, "pressed": n.pressed, "selected": n.selected,
            "checked": n.checked, "locked": n.locked, "actions": n.actions, "longClickLabel": n.long_click_label,
            "holdDelayMillis": n.hold_delay(&s.scene, s.touch.long_press_delay / 1_000_000),
            "after": n.after, "layer": n.layer,
        })).collect();
        let value = json!({"nodes": nodes, "mode": s.semantics.labels.mode(&s.mode),
            "shift": s.semantics.labels.shift(&s.shift), "popup": s.scene.panel.as_ref().is_some_and(|p|p.kind==7) || s.scene.panel.as_ref().and_then(|p|p.clipboard.as_ref()).is_some_and(|c|c.modal()) || s.scene.popup.is_some() || s.scene.panel.as_ref().and_then(|p|p.emoji.as_ref()).is_some_and(|e|e.popup.is_some())});
        Ok(value.to_string().encode_utf16().collect::<Vec<_>>())
    })
}
pub fn semantic_action(native: i64, id: i32, action: i32) -> Result<Vec<i32>> {
    session(native, |s| {
        let actions: Vec<_> = s
            .semantics
            .activate(&mut s.scene, &s.shift, id, action)
            .into_iter()
            .flat_map(|a| [a.0, a.1, a.2])
            .collect();
        s.prepare_panel();
        Ok(actions)
    })
}
/// The label rules alone, for the JVM tests. Only app/src/test declares it, so the phone's library
/// leaves it out (scripts/native-declarations-check.py).
#[cfg(not(target_os = "android"))]
mod probe {
    use crate::{
        jni::{self, illegal},
        render::strings,
    };
    use jni_sys::{JNIEnv, jclass, jint, jintArray, jobjectArray, jstring};
    use render::accessibility_labels::Labels;
    use std::ptr;

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
}
