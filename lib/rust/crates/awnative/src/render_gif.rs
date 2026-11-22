//! GIF results metadata; network and decoding remain at the Android edge.
use crate::{
    jni::{Result, illegal},
    render::{session, texts},
};
use render::gif_panel::{GifEntry, GifPanel};
pub fn gifs(
    native: i64,
    config: &[f32],
    labels: &[Vec<u16>],
    chips: i32,
    ids: &[i64],
) -> Result<()> {
    session(native, |s| {
        let c = config;
        let labels = texts(labels);
        if c.len() < 2
            || ids.len() != c.len() - 2
            || chips < 0
            || labels.len() != 6 + chips as usize + c.len() - 2
            || c.iter().any(|v| !v.is_finite())
        {
            return Err(illegal("gif snapshot"));
        }
        let Some(p) = &mut s.scene.panel else {
            return Ok(());
        };
        let entries: Vec<_> = c[2..]
            .iter()
            .enumerate()
            .map(|(i, a)| GifEntry {
                id: ids[i],
                label: labels[6 + chips as usize + i].clone(),
                aspect: *a,
            })
            .collect();
        let next = GifPanel {
            entries,
            chips: labels[6..6 + chips as usize].to_vec(),
            shown: labels[0].clone(),
            labels: labels[1..6].to_vec(),
            status: c[0] as i32,
            bottom: c[1],
            ..Default::default()
        };
        if let Some(old) = &p.gif {
            if old.shown == next.shown && old.entries == next.entries {
                let mut n = next;
                n.sending = old.sending;
                n.chip_scroll = old.chip_scroll;
                p.gif = Some(n);
            } else {
                p.gif = Some(next);
                p.scroll = 0.0;
                p.media = Default::default();
            }
        } else {
            p.gif = Some(next);
        }
        s.prepare_panel();
        Ok(())
    })
}

pub fn gif_sent(native: i64) -> Result<()> {
    session(native, |s| {
        if let Some(p) = &mut s.scene.panel
            && let Some(g) = &mut p.gif
        {
            g.sending = None;
        }
        s.prepare_panel();
        Ok(())
    })
}
