//! Native search strip metadata; query editing and InputConnection routing use the existing dispatcher.
use crate::{
    handle,
    jni::illegal,
    render::{Session, strings},
};
use jni_sys::{JNIEnv, jclass, jfloat, jlong, jobjectArray};
use render::{gif_search::Search, panels::Panel};
impl Session {
    pub(crate) fn prepare_search(&mut self) {
        let Some(p) = &mut self.scene.search else {
            return;
        };
        let r = render::theme::Request {
            element: String::new(),
            mode: self.mode.clone(),
            shift: self.shift.clone(),
            code: 0,
            key_type: String::new(),
            selector: 0,
            auto_commit: false,
            popup_source: None,
        };
        p.row = self.scene.strip;
        p.search_layout(self.scene.width as f32);
        if let (Some(theme), Some(text)) = (&mut self.theme, &mut self.text) {
            p.style(theme, &r);
            p.search_style(theme, text, &r);
        }
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_search(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    density: jfloat,
    labels: jobjectArray,
    now: jlong,
) {
    // SAFETY: synchronized owner retains the session and the copied strings contain no JVM references.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let labels = strings(env, labels)?;
            if labels.is_empty() {
                s.scene.search = None;
                return Ok(());
            }
            if labels.len() != 4 || !density.is_finite() || density <= 0.0 {
                return Err(illegal("search strip"));
            }
            let search = Search {
                query: labels[0].clone(),
                back: labels[1].clone(),
                prompt: labels[2].clone(),
                submit: labels[3].clone(),
                cursor: true,
                deadline: now + 530_000_000,
            };
            if let Some(p) = &mut s.scene.search {
                if p.search
                    .as_ref()
                    .is_none_or(|old| old.query != search.query)
                {
                    p.search = Some(search);
                }
            } else {
                s.scene.search = Some(Panel {
                    kind: 6,
                    density,
                    search: Some(search),
                    ..Default::default()
                });
            }
            s.scene.smartbar = None;
            s.scene.candidates.clear();
            s.prepare_search();
            Ok(())
        });
    }
}
