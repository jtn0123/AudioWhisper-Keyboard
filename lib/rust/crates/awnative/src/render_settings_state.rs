//! Versioned saved-state transport. No preference values or native handles are restored.
use render::settings_ui::{
    App, Modal,
    catalogue::{KEYBOARD, Kind},
};
use settings::{Text, text};
pub fn save(a: &App) -> Vec<Text> {
    let m = a.modal.as_ref();
    vec![
        text("1"),
        text(&a.stack.join("|")),
        text(&a.scrolls[0].to_string()),
        text(&a.scrolls[1].to_string()),
        text(&m.map_or(-1, |m| m.row as i32).to_string()),
        text(&m.map_or(String::new(), |m| m.selected.clone())),
        text(&m.map_or(String::new(), |m| m.values.join("|"))),
        text(&m.map_or(0.0, |m| m.scroll).to_string()),
        a.editor.text.clone(),
        text(&a.editor.anchor.to_string()),
        text(&a.editor.focus.to_string()),
        text(&a.editor.composing.map_or(-1, |(s, _)| s as i32).to_string()),
        text(&a.editor.composing.map_or(-1, |(_, e)| e as i32).to_string()),
        text(&a.editor.focused.to_string()),
        text(&a.focus.to_string()),
        text(&m.is_none_or(|m| m.enabled).to_string()),
    ]
}
pub fn restore(a: &mut App, packet: &[Text]) -> Result<(), &'static str> {
    if packet.len() != 16 || packet[0] != text("1") || packet[8].len() > 65_536 {
        return Err("settings saved state version or size");
    }
    let p: Vec<_> = packet.iter().map(|s| String::from_utf16_lossy(s)).collect();
    let number = |i: usize| p[i].parse::<i32>().map_err(|_| "settings saved integer");
    let float = |i: usize| {
        p[i].parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .ok_or("settings saved scroll")
    };
    let stack: Vec<_> = p[1].split('|').map(String::from).collect();
    if stack.is_empty()
        || stack.len() > 32
        || stack
            .iter()
            .any(|r| !matches!(r.as_str(), "settings/home" | "settings/keyboard"))
    {
        return Err("settings saved route");
    }
    let scrolls = [float(2)?, float(3)?];
    let row = number(4)?;
    let modal = if row < 0 {
        None
    } else {
        let row = row as usize;
        let r = KEYBOARD.get(row).ok_or("settings saved modal")?;
        let values: Vec<_> = p[6].split('|').map(String::from).collect();
        match r.kind {
            Kind::Choice(_) if App::choices(row).contains(&p[5]) && values.len() == 1 => {}
            Kind::Slider(second, _)
                if values.len() == if second.is_empty() { 1 } else { 2 }
                    && values
                        .iter()
                        .all(|s| s.parse::<f32>().is_ok_and(f32::is_finite)) => {}
            _ => return Err("settings saved modal values"),
        }
        Some(Modal {
            row,
            selected: p[5].clone(),
            values,
            scroll: float(7)?,
            enabled: p[15].parse().map_err(|_| "settings saved switch")?,
        })
    };
    let (anchor, focus) = (number(9)?, number(10)?);
    let (start, end) = (number(11)?, number(12)?);
    if anchor < 0
        || focus < 0
        || anchor as usize > packet[8].len()
        || focus as usize > packet[8].len()
        || !((start == -1 && end == -1)
            || (start >= 0 && end >= start && end as usize <= packet[8].len()))
    {
        return Err("settings saved editor bounds");
    }
    let focused = p[13].parse::<bool>().map_err(|_| "settings saved focus")?;
    let id = number(14)?;
    a.stack = stack;
    a.scrolls = scrolls;
    a.modal = modal;
    a.editor.text = packet[8].clone();
    a.editor.anchor = anchor as usize;
    a.editor.focus = focus as usize;
    a.editor.composing = (start >= 0).then_some((start as usize, end as usize));
    a.editor.focused = focused;
    a.focus = id;
    a.fling = None;
    a.down = None;
    Ok(())
}
