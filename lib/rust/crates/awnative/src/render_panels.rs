//! Panel payload and preference transport. Kotlin supplies data, Rust owns panel geometry/state.
use crate::{
    jni::{Result, illegal},
    render::{Session, session, texts},
};
use render::{
    arrangement::Arrangement,
    panels::{Entry, Panel},
};
impl Session {
    pub(crate) fn prepare_panel(&mut self) {
        if self.settings.is_some() {
            return;
        }
        self.prepare_search();
        let Some(p) = &mut self.scene.panel else {
            return;
        };
        let request = render::theme::Request {
            element: String::new(),
            mode: self.mode.clone(),
            shift: self.shift.clone(),
            code: 0,
            key_type: String::new(),
            selector: 0,
            auto_commit: false,
            popup_source: None,
        };
        if let (Some(theme), Some(text)) = (&mut self.theme, &mut self.text) {
            p.measure(theme, text, &request, self.scene.width as f32);
        }
        p.layout(
            self.scene.width as f32,
            self.scene.height as f32,
            self.scene.strip,
        );
        let visible = p
            .items
            .iter()
            .filter(|i| i.bounds.bottom > i.clip.top && i.bounds.top < i.clip.bottom)
            .filter_map(|i| i.media)
            .collect();
        p.media.visibility(visible, self.touch_time);
        if let Some(theme) = &mut self.theme {
            p.style(theme, &request);
        }
    }
}
pub fn panels(native: i64, config: &[f32], data: &[i32], labels: &[Vec<u16>]) -> Result<()> {
    session(native, |s| {
        let c = config;
        let a = data;
        let labels = texts(labels);
        if c.len() != 7 || c.iter().any(|v| !v.is_finite()) || a.len() < 3 {
            return Err(illegal("panel payload"));
        }
        let kind = c[0] as i32;
        if s.scene.panel.as_ref().is_some_and(|p| p.kind != kind) {
            s.touch.cancel(&mut s.scene);
        }
        if kind == 0 {
            s.scene.panel = None;
            return Ok(());
        }
        let count = a[1].max(0) as usize;
        let hidden = a[2].max(0) as usize;
        if a.len() < 3 + count + hidden {
            return Err(illegal("panel arrangement"));
        }
        let arrangement = Arrangement {
            sticky: a[0],
            dynamic: a[3..3 + count].to_vec(),
            hidden: a[3 + count..3 + count + hidden].to_vec(),
        };
        let actions = &a[3 + count + hidden..];
        if !actions.len().is_multiple_of(3) || labels.len() < 6 + actions.len() {
            return Err(illegal("panel entries"));
        }
        let entries = actions
            .as_chunks::<3>()
            .0
            .iter()
            .enumerate()
            .map(|(i, v)| Entry {
                id: v[0],
                code: v[1],
                enabled: v[2] & 1 != 0,
                insert_text: v[2] & 2 != 0,
                label: labels[6 + i * 3].clone(),
                glyph: labels[7 + i * 3].clone(),
                icon: labels[8 + i * 3].clone(),
            })
            .collect();
        if let Some(p) = &mut s.scene.panel {
            p.set_kind(kind);
            p.entries = entries;
            p.arrangement = arrangement;
            p.visible_actions = c[3].max(0.0) as usize;
            p.titles = labels[..6].to_vec();
            p.density = c[1];
            p.row = c[2];
            p.repeat_delay = c[4] as i64;
            p.gesture_delay = c[6] as i64;
            p.delete_words = c[5] != 0.0;
        } else {
            let editor = (kind == 3).then(|| render::arrangement::Editor::new(arrangement.clone()));
            s.scene.panel = Some(Panel {
                kind,
                emoji: None,
                density: c[1],
                row: c[2],
                repeat_delay: c[4] as i64,
                gesture_delay: c[6] as i64,
                delete_words: c[5] != 0.0,
                visible_actions: c[3].max(0.0) as usize,
                entries,
                arrangement,
                editor,
                scroll: 0.0,
                overflow_scroll: 0.0,
                emoji_header: 0.0,
                message_heights: [0.0; 2],
                extent: 0.0,
                subheaders: [0.0; 3],
                grid_margin: [0.0; 4],
                max_scroll: 0.0,
                items: vec![],
                titles: labels[..6].to_vec(),
                pressed: None,
                drag_position: [0.0; 2],
                ..Panel::default()
            });
        }
        s.prepare_panel();
        Ok(())
    })
}
pub fn panel_arrangement(native: i64) -> Result<Vec<Vec<i32>>> {
    session(native, |s| {
        let p = s
            .scene
            .panel
            .as_mut()
            .ok_or_else(|| illegal("panel missing"))?;
        let a = p
            .editor
            .as_mut()
            .map(|e| e.saved())
            .unwrap_or_else(|| p.arrangement.clone());
        Ok(vec![vec![a.sticky], a.dynamic, a.hidden])
    })
}
