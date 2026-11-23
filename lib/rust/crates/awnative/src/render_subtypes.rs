//! Subtype resource transport; order, selection, layout and input are native.
use crate::{
    jni::{Result, illegal},
    render::{session, texts},
};
pub fn subtypes(
    native: i64,
    selected: i64,
    ids: &[i64],
    labels: &[Vec<u16>],
    bottom_inset: f32,
) -> Result<()> {
    session(native, |s| {
        let labels = texts(labels);
        if labels.len() != ids.len() + 1 || !bottom_inset.is_finite() || bottom_inset < 0.0 {
            return Err(illegal("subtype labels"));
        }
        let p = s
            .scene
            .panel
            .as_mut()
            .ok_or_else(|| illegal("subtype panel missing"))?;
        if p.kind != 7 {
            return Err(illegal("wrong subtype panel"));
        }
        p.subtypes = Some(render::subtype_panel::Subtypes {
            ids: ids.to_vec(),
            selected,
            bottom_inset,
            header: labels[0].clone(),
            icons: icons::SUBTYPES.map(|n| icons::path(n).unwrap_or_default().to_owned()),
            labels: labels[1..].to_vec(),
            ..Default::default()
        });
        s.prepare_panel();
        Ok(())
    })
}
pub fn rows(selected: i64, ids: &[i64], labels: &[Vec<u16>]) -> Result<Vec<Vec<u16>>> {
    let labels = texts(labels);
    if ids.len() != labels.len() {
        return Err(illegal("subtype count"));
    }
    let out: Vec<Vec<u16>> = render::subtype_panel::rows(ids, &labels, selected)
        .into_iter()
        .flat_map(|(id, label, selected)| [id.to_string(), label, selected.to_string()])
        .map(|s| s.encode_utf16().collect())
        .collect();
    Ok(out)
}

pub fn geometry(c: &[f32]) -> Result<Vec<f32>> {
    if c.len() != 7 || c.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(illegal("subtype geometry"));
    }
    Ok(vec![
        render::subtype_panel::sheet_start(
            render::subtype_panel::content_height(c[0], c[6]),
            c[1],
            c[2],
            c[3] as usize,
        ),
        render::modal_scene::offset(c[0], c[4], c[5]),
    ])
}
