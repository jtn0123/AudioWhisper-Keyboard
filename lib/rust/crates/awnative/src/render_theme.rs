//! Raw stylesheet/palette boundary and, off the phone, the test-only JVM reference probe.
use crate::jni::{Result, illegal};
use render::{style::color, theme::Theme};
use snygg::text;

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

/// The theme's `@font` files from Kotlin's flat array: each `src` URI followed by its absolute
/// path. The paths are only read, by `render::theme_fonts`; neither is logged.
pub(crate) fn font_files(flat: &[String]) -> Result<Vec<(String, String)>> {
    if !flat.len().is_multiple_of(2) {
        return Err(illegal("invalid theme font files"));
    }
    Ok(flat
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[uri, path]| (uri.clone(), path.clone()))
        .collect())
}

/// The JVM reference probe. Only lib/snygg/src/test declares it, so the phone's library leaves it
/// out (scripts/native-declarations-check.py).
#[cfg(not(target_os = "android"))]
mod probe {
    use super::theme;
    use crate::{
        jni::{self, illegal},
        render::strings,
    };
    use jni_sys::{JNIEnv, jclass, jfloatArray, jintArray, jobjectArray, jstring};
    use render::theme::Request;
    use snygg::text;
    use std::ptr;

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
}

#[cfg(test)]
mod tests {
    use super::font_files;
    use crate::{render, render_chrome};
    use std::path::PathBuf;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    /// The session draws the theme's `@font` files Kotlin passes, and a broken one as none: the
    /// landscape extract view's action label, which the session lays out and rasters offscreen.
    #[test]
    fn the_session_draws_a_label_in_the_font_files_it_is_given() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
        let roboto = root.join("fastlane/staging/fonts/Roboto-Light.ttf");
        let script =
            root.join("lib/rust/crates/render/tests/fixtures/fixture-script-regular-subset.ttf");
        let native = render::create(&[utf16(&roboto.display().to_string())]).unwrap();
        let json = r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
            "@font `F`":[{"src":"uri(`flex:/f.ttf`)"}],
            "extracted-landscape-input-action":{"background":"#ffffff","foreground":"#000000",
                "font-family":"`F`","font-size":"24sp"}}"##;
        let preview = |path: Option<&str>| {
            let fonts: Vec<_> = path.map_or(vec![], |p| vec![utf16("flex:/f.ttf"), utf16(p)]);
            render::theme(native, &utf16(json), &[], &[], &[2.0, 1.0, 1.0], &fonts).unwrap();
            let state = ["characters", "unshifted"].map(utf16);
            render_chrome::chrome(
                native,
                &[400.0, 120.0, 2.0, 1.0, 0.0],
                &[utf16("Hello")],
                &state,
            )
            .unwrap();
            render_chrome::pixels(native).unwrap()
        };
        let plain = preview(None);
        let themed = preview(Some(&script.display().to_string()));
        let differing = plain.iter().zip(&themed).filter(|(a, b)| a != b).count();
        assert!(differing > 200, "the theme font changed {differing} pixels");
        assert_eq!(preview(Some("/nonexistent/theme-font.ttf")), plain);
        render::close(native).unwrap();
    }

    #[test]
    fn font_files_pair_each_uri_with_its_path_and_refuse_a_lone_one() {
        let flat = ["flex:/a.ttf", "/data/a.ttf", "flex:/b.ttf", "/data/b.ttf"].map(String::from);
        let pairs = font_files(&flat).unwrap();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[1], ("flex:/b.ttf".into(), "/data/b.ttf".into()));
        assert!(font_files(&flat[..3]).is_err());
        assert!(font_files(&[]).unwrap().is_empty());
    }
}
