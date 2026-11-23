//! Resource/preview transport; never logs text or retains Java references.
use crate::{
    jni::{Result, illegal},
    render::{session, texts},
};
pub fn decorations(native: i64, labels: &[Vec<u16>], height: f32, flags: &[i32]) -> Result<()> {
    session(native, |s| {
        let labels = texts(labels);
        if flags.len() != 2 {
            return Err(illegal("decoration flags"));
        }
        if labels.len() != 1 || !height.is_finite() || height <= 0.0 {
            return Err(illegal("decorations"));
        }
        let request = render::theme::Request {
            element: String::new(),
            mode: s.mode.clone(),
            shift: s.shift.clone(),
            code: 0,
            key_type: String::new(),
            selector: 0,
            auto_commit: false,
            popup_source: None,
        };
        if let (Some(theme), Some(text)) = (&mut s.theme, &mut s.text) {
            render::ime_decorations::prepare(
                &mut s.scene,
                theme,
                text,
                &request,
                if render::ime_decorations::show_watermark(flags[0] != 0, flags[1] != 0) {
                    icons::path(icons::WATERMARK).unwrap_or_default()
                } else {
                    ""
                },
                &labels[0],
                height,
            );
        }
        Ok(())
    })
}
pub fn watermark(flags: &[i32]) -> Result<bool> {
    if flags.len() != 2 {
        return Err(illegal("watermark flags"));
    }
    Ok(render::ime_decorations::show_watermark(
        flags[0] != 0,
        flags[1] != 0,
    ))
}
