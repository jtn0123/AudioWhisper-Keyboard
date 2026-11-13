//! Raw stylesheet/palette boundary and the test-only JVM reference probe.
use crate::{
    jni::{self, Result, illegal},
    render::strings,
};
use jni_sys::{JNIEnv, jclass, jfloatArray, jintArray, jobjectArray, jstring};
use render::{
    style::color,
    theme::{Request, Theme},
};
use snygg::text;
use std::ptr;

pub(crate) fn theme(json: &str, names: &[String], colors: &[i32], scale: &[f32]) -> Result<Theme> {
    if colors.len() != names.len() * 2 || scale.len() != 3 {
        return Err(illegal("invalid theme arrays"));
    }
    let palette = |offset| {
        names
            .iter()
            .enumerate()
            .map(|(i, name)| (text(name), color(colors[offset + i] as u32)))
            .collect()
    };
    Theme::new(
        json,
        palette(0),
        palette(names.len()),
        scale[0],
        scale[1],
        scale[2],
    )
    .map_err(|_| illegal("invalid renderer stylesheet or scale"))
}

// The declaration lives only in lib/snygg/src/test. R8 has no production call site.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_florisboard_lib_snygg_RenderThemeProbe_resolve(
    env: *mut JNIEnv,
    _: jclass,
    json: jstring,
    names: jobjectArray,
    colors: jintArray,
    scale: jfloatArray,
    queries: jobjectArray,
) -> jobjectArray {
    // SAFETY: JVM owns the environment and local arrays for the duration of this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let json = String::from_utf16(&env.read_string(json, "stylesheet")?)
                .map_err(|_| illegal("invalid stylesheet UTF-16"))?;
            let mut theme = theme(
                &json,
                &strings(env, names)?,
                &env.read::<i32>(colors, "palette")?,
                &env.read::<f32>(scale, "scale")?,
            )?;
            let mut output = vec![];
            for query in strings(env, queries)? {
                let parts: Vec<_> = query.split('\t').collect();
                if !(7..=8).contains(&parts.len()) {
                    return Err(illegal("invalid theme query"));
                }
                let request = Request {
                    element: parts[0].into(),
                    mode: parts[1].into(),
                    shift: parts[2].into(),
                    code: parts[3].parse().map_err(|_| illegal("code"))?,
                    key_type: parts[4].into(),
                    selector: parts[5].parse().map_err(|_| illegal("selector"))?,
                    auto_commit: parts[6] == "1",
                    popup_source: parts
                        .get(7)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.parse())
                        .transpose()
                        .map_err(|_| illegal("popup source"))?,
                };
                let properties = theme.properties(&request);
                output.push(text(&properties.len().to_string()));
                for (key, value) in properties {
                    output.push(key);
                    output.push(
                        value
                            .encoder()
                            .serialize(&value)
                            .map_err(|_| illegal("style value"))?,
                    );
                }
            }
            env.make_strings(&output)
        })
    }
}
