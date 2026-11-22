//! Native search strip metadata; query editing and InputConnection routing use the existing dispatcher.
use crate::{
    jni::{Result, illegal},
    render::{Session, session, texts},
};
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
pub fn search(native: i64, density: f32, labels: &[Vec<u16>], now: i64) -> Result<()> {
    session(native, |s| {
        let labels = texts(labels);
        if labels.is_empty() {
            s.scene.search = None;
            return Ok(());
        }
        if labels.len() != 6 || !density.is_finite() || density <= 0.0 {
            return Err(illegal("search strip"));
        }
        let search = Search {
            query: labels[0].clone(),
            back: labels[1].clone(),
            prompt: labels[2].clone(),
            submit: labels[3].clone(),
            back_icon: labels[4].clone(),
            submit_icon: labels[5].clone(),
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
    })
}
