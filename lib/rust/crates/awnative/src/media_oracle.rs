//! Differential probes use the same layout, scheduler and budget code as the live surface.
use crate::jni::{Result, illegal};
use render::media_cache::{Frame, MediaCache};

pub fn trace(c: &[i32], ops: &[i64]) -> Result<Vec<i64>> {
    if c.len() != 2 || c.iter().any(|v| *v < 0) || !ops.len().is_multiple_of(8) {
        return Err(illegal("media trace"));
    }
    let mut cache = MediaCache::default();
    cache.budget = c[0] as usize;
    cache.per_item = c[1] as usize;
    let mut out = vec![];
    for o in ops.as_chunks::<8>().0 {
        let mut accepted = false;
        match o[0] {
            0 => {
                if !(1..=32).contains(&o[4]) || !(1..=32).contains(&o[5]) || o[2] < 0 {
                    return Err(illegal("trace dimensions"));
                }
                let f = Frame::argb(
                    o[4] as u16,
                    o[5] as u16,
                    &vec![0; o[4] as usize * o[5] as usize],
                    o[3],
                )
                .unwrap();
                accepted = cache.upload(o[1], o[2] as usize, f, o[6] != 0);
            }
            1 => cache.visibility(
                if o[1] == 0 {
                    Default::default()
                } else {
                    [o[1]].into()
                },
                o[7],
            ),
            2 => {
                accepted = cache.tick(o[7]);
            }
            3 => cache = MediaCache::default(),
            4 => cache.repeat(o[1], o[2] as i32),
            _ => return Err(illegal("trace operation")),
        }
        out.extend([
            cache.entries.get(&o[1]).map_or(-1, |e| e.frame as i64),
            cache.deadline().unwrap_or(-1),
            cache.bytes as i64,
            cache.entries.len() as i64,
            i64::from(accepted),
        ]);
    }
    Ok(out)
}
pub fn grid(c: &[f32]) -> Result<Vec<f32>> {
    if c.len() < 2 || c.iter().any(|v| !v.is_finite()) || c[0] <= 0.0 || c[1] <= 0.0 {
        return Err(illegal("gif grid"));
    }
    let rects = render::gif_panel::grid(c[0], c[1], &c[2..]);
    Ok(rects
        .iter()
        .flat_map(|r| [r.left, r.top, r.right, r.bottom])
        .collect::<Vec<_>>())
}

pub fn scroll_chips(c: &[f32]) -> Result<Vec<f32>> {
    if c.len() != 4 || c.iter().any(|v| !v.is_finite()) {
        return Err(illegal("chip scroll"));
    }
    Ok(vec![render::gif_panel::scroll_chips(
        c[0],
        c[1],
        c[2],
        c[3] != 0.0,
    )])
}
