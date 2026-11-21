//! Font-independent oracle for the phone-key content placement used by the renderer.
use crate::jni::{self, illegal};
use jni_sys::{JNIEnv, jboolean, jfloatArray, jobject, jstring};
use std::ptr;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderContentRules_phoneLabel(
    env: *mut JNIEnv,
    _: jobject,
    mode: jstring,
    kind: jstring,
    values: jfloatArray,
    hint: jboolean,
) -> jfloatArray {
    // SAFETY: JVM local strings/array are copied during this call; the result is a new local array.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let mode = String::from_utf16_lossy(&env.read_string(mode, "mode")?);
            let kind = String::from_utf16_lossy(&env.read_string(kind, "kind")?);
            let v = env.read::<f32>(values, "content bounds")?;
            if v.len() != 6 || v.iter().any(|x| !x.is_finite()) {
                return Err(illegal("invalid content bounds"));
            }
            let area = keys::Rect {
                left: v[0],
                top: v[1],
                right: v[2],
                bottom: v[3],
            };
            let r = if render::key_content::telpad(&mode, &kind) {
                render::key_content::phone_label(area, [v[4], v[5]], hint != 0)
            } else {
                area
            };
            env.make(&[r.left, r.top, r.right, r.bottom])
        })
    }
}
