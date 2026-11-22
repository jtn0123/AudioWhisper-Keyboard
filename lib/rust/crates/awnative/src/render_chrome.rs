//! Offscreen Vello CPU drawing for small chrome views and settings images.
use crate::{
    jni::{Result, illegal},
    render::{session, texts},
};
pub fn chrome(
    native: i64,
    config: &[f32],
    labels: &[Vec<u16>],
    icons: &[Vec<u16>],
    metadata: &[Vec<u16>],
) -> Result<()> {
    session(native, |s| {
        let c = config;
        let metadata = texts(metadata);
        if metadata.len() != 2 {
            return Err(illegal("chrome metadata"));
        }
        s.mode = metadata[0].clone();
        s.shift = metadata[1].clone();
        if c.len() != 4
            || c.iter().any(|v| !v.is_finite())
            || !(1.0..=8192.0).contains(&c[0])
            || !(1.0..=8192.0).contains(&c[1])
            || c[2] <= 0.0
        {
            return Err(illegal("chrome dimensions"));
        }
        if s.scene.width != c[0] as u16 || s.scene.height != c[1] as u16 {
            s.bitmap = None;
        }
        s.scene.width = c[0] as u16;
        s.scene.height = c[1] as u16;
        s.scene.strip = 0.0;
        s.scene.panel = Some(render::panels::Panel {
            kind: 8,
            density: c[2],
            chrome: Some(render::ime_chrome::Chrome {
                kind: c[3] as i32,
                labels: texts(labels),
                icons: texts(icons),
                ..Default::default()
            }),
            ..Default::default()
        });
        s.prepare_panel();
        Ok(())
    })
}
pub fn pixels(native: i64) -> Result<Vec<i32>> {
    session(native, |s| {
        s.prepare_panel();
        let raster = s.bitmap.get_or_insert_with(|| {
            render::raster::Raster::shared(s.scene.width, s.scene.height, s.fonts.clone())
        });
        raster
            .draw(
                &s.scene,
                keys::Rect {
                    left: 0.0,
                    top: 0.0,
                    right: s.scene.width as f32,
                    bottom: s.scene.height as f32,
                },
            )
            .map_err(|_| illegal("chrome raster failed"))?;
        let pixels = argb(raster.pixels());
        Ok(pixels)
    })
}
pub fn one_handed(c: &[i32]) -> Result<Vec<f32>> {
    if c.len() != 4 {
        return Err(illegal("one-handed config"));
    }
    Ok(render::ime_chrome::one_handed(c[0] != 0, c[1] != 0, c[2], c[3] != 0).to_vec())
}
pub fn side_buttons(c: &[f32]) -> Result<Vec<f32>> {
    if c.len() != 2 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(illegal("side dimensions"));
    }
    Ok(render::ime_chrome::side_buttons(c[0], c[1])
        .iter()
        .flat_map(|r| [r.left, r.top, r.right, r.bottom])
        .collect::<Vec<_>>())
}

pub fn chrome_info(native: i64) -> Result<Vec<f32>> {
    session(native, |s| {
        let c = s
            .scene
            .panel
            .as_ref()
            .and_then(|p| p.chrome.as_ref())
            .ok_or_else(|| illegal("chrome missing"))?;
        Ok(c.field.to_vec())
    })
}
pub fn extract(c: &[f32]) -> Result<Vec<f32>> {
    if c.len() != 5 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(illegal("extract config"));
    }
    Ok(
        render::extract_chrome::geometry(c[0], c[1], c[2], [c[3], c[4]])
            .iter()
            .flat_map(|r| [r.left, r.top, r.right, r.bottom])
            .collect::<Vec<_>>(),
    )
}

pub(crate) fn argb(pixels: &[u8]) -> Vec<i32> {
    pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            let a = p[3] as u32;
            let channel = |v: u8| {
                (v as u32 * 255 + a / 2)
                    .checked_div(a)
                    .map_or(0, |n| n.min(255))
            };
            ((a << 24) | (channel(p[0]) << 16) | (channel(p[1]) << 8) | channel(p[2])) as i32
        })
        .collect()
}

pub fn preview_size(c: &[f32]) -> Result<Vec<i32>> {
    if c.len() != 2 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(illegal("preview config"));
    }
    Ok(render::key_preview::size(c[0], c[1])
        .map(i32::from)
        .to_vec())
}
pub fn start_ellipsis(labels: &[Vec<u16>], c: &[f32]) -> Result<Vec<Vec<u16>>> {
    let labels = texts(labels);
    if c.len() != 2 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(illegal("ellipsis config"));
    }
    let output: Vec<Vec<u16>> = labels
        .iter()
        .map(|label| {
            render::ime_decorations::start_ellipsis(label, c[0], |s| {
                s.encode_utf16().count() as f32 * c[1]
            })
            .encode_utf16()
            .collect()
        })
        .collect();
    Ok(output)
}

pub fn minimum_target(c: &[f32]) -> Result<Vec<f32>> {
    if c.len() != 6 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(illegal("minimum target"));
    }
    let header = c[5] == 0.0;
    let mut item = render::panels::Item::new(
        1,
        render::panels::rect(0.0, c[2], c[1], c[3]),
        "label".into(),
        if header {
            "subtype-panel-header"
        } else {
            "extracted-landscape-input-action"
        },
        render::panels::Command::None,
    );
    item.clip = render::panels::rect(0.0, 0.0, c[1], 10000.0);
    let mut items = vec![item];
    if header {
        items.push(render::panels::Item::new(
            2,
            render::panels::rect(0.0, c[4], c[1], 56.0 * c[0]),
            "row".into(),
            "subtype-panel-list-item",
            render::panels::Command::Subtype(Some(0)),
        ));
    }
    let panel = render::panels::Panel {
        density: c[0],
        items,
        ..Default::default()
    };
    let r = panel.target_bounds(0);
    Ok(vec![r.left, r.top, r.right, r.bottom])
}

pub fn font_multiplier(c: &[i32]) -> Result<f32> {
    if c.len() != 4 {
        return Err(illegal("font multiplier"));
    }
    Ok(render::ime_chrome::font_multiplier(
        c[0],
        c[1],
        c[2] != 0,
        c[3] != 0,
    ))
}
pub fn row_heights(base: f32, c: &[i32]) -> Result<Vec<f32>> {
    if c.len() != 5 || !base.is_finite() || base < 0.0 {
        return Err(illegal("row heights"));
    }
    Ok(render::ime_chrome::row_heights(base, c[0], c[1], c[2], c[3] != 0, c[4] != 0).to_vec())
}
