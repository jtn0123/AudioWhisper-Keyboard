//! Clipboard data transport. Android retains Room objects; native retains display snapshots only.
use crate::{
    jni::{Result, illegal},
    render::{session, texts},
};
use render::clipboard_panel::{ClipEntry, ClipboardPanel};
pub fn clipboard(native: i64, config: &[i64], records: &[i64], labels: &[Vec<u16>]) -> Result<()> {
    session(native, |s| {
        let c = config;
        let a = records;
        let labels = texts(labels);
        if c.len() != 4 || !a.len().is_multiple_of(4) || labels.len() != 21 + a.len() / 4 * 3 {
            return Err(illegal("clipboard snapshot"));
        }
        let next = ClipboardPanel {
            now: c[0],
            enabled: c[1] != 0,
            locked: c[2] != 0,
            columns: c[3].max(0) as usize,
            labels: labels[..21].to_vec(),
            icons: icons::owned(&icons::CLIPBOARD),
            entries: a
                .as_chunks::<4>()
                .0
                .iter()
                .enumerate()
                .map(|(i, e)| ClipEntry {
                    id: e[0],
                    semantic: 0,
                    timestamp: e[1],
                    kind: e[2] as i32,
                    pinned: e[3] != 0,
                    label: labels[21 + i * 3].clone(),
                    description: labels[22 + i * 3].clone(),
                    date: labels[23 + i * 3].clone(),
                    aspect: 1.0,
                    height: 0.0,
                    date_height: 0.0,
                    full_height: 0.0,
                })
                .collect(),
            ..Default::default()
        };
        if let Some(p) = &mut s.scene.panel {
            if let Some(c) = &mut p.clipboard {
                c.update(next);
            } else {
                let mut c = ClipboardPanel::default();
                c.update(next);
                p.clipboard = Some(c);
            }
        }
        s.prepare_panel();
        Ok(())
    })
}
