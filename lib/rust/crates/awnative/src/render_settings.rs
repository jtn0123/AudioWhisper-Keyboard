//! JNI copies only: settings policy/geometry/editor live in the unsafe-free renderer module.
use crate::{
    jni::{Result, illegal},
    render::{Session, session},
};
use render::{
    settings_ui::{Action, App, Control, catalogue},
    touch::Motion,
};
use serde_json::json;
use wire::render::{ACCESS_CLICK, ACCESS_SCROLL_BACKWARD, ACCESS_SCROLL_FORWARD};
use wire::settings_ui::{self as ops, back, edit, query, reply};
fn field(p: &[String], i: usize) -> Result<&str> {
    p.get(i)
        .map(String::as_str)
        .ok_or_else(|| illegal("missing settings field"))
}
fn int(p: &[String], i: usize) -> Result<i32> {
    field(p, i)?
        .parse()
        .map_err(|_| illegal("settings integer"))
}
fn float(p: &[String], i: usize) -> Result<f32> {
    field(p, i)?
        .parse::<f32>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| illegal("settings number"))
}
/// A node's checked state: a radio button's selection, a checkbox's with "partial" when it is
/// partly on, or null for a control that cannot be checked.
fn checked(c: &Control) -> serde_json::Value {
    match (c.role, c.checked) {
        ("radio", _) => c.selected.into(),
        ("checkbox", None) => "partial".into(),
        (_, state) => state.map_or(serde_json::Value::Null, Into::into),
    }
}
fn reply(s: &mut Session, query: Vec<u16>, saved: bool) -> Result<Vec<Vec<u16>>> {
    let a = s
        .settings
        .as_mut()
        .ok_or_else(|| illegal("settings missing"))?;
    let text = s
        .text
        .as_mut()
        .ok_or_else(|| illegal("settings fonts missing"))?;
    #[cfg(target_os = "android")]
    let _trace = crate::render_profile::Span::new(c"aw.settings.scene_reply");
    s.scene = a.scene(text);
    // A drag surface (the colour plane and hue bar) has no node, as Compose's pointerInput has none.
    let nodes:Vec<_>=a.controls.iter().filter(|c| c.role!="none").map(|c|{
        let mut actions=if !c.enabled || matches!(c.action,Action::None){vec![]}else{vec![ACCESS_CLICK]};
        if c.enabled && c.role=="slider" {actions.extend([ACCESS_SCROLL_FORWARD,ACCESS_SCROLL_BACKWARD]);} else if c.role=="scroll" {let [back,forward]=a.node_can_scroll(c.id); if forward {actions.push(ACCESS_SCROLL_FORWARD);}
            if back {actions.push(ACCESS_SCROLL_BACKWARD);}}
        // An image has a content description and no text, as Compose's Image does.
        let edited=(c.role=="edit").then(|| a.field_editor(c.id).0);
        let shown=edited.as_ref().map(|e| String::from_utf16_lossy(&e.text));
        let (anchor,focus)=edited.as_ref().map_or_else(|| a.node_selection(c.id),|e| (e.anchor as i64,e.focus as i64));
        let label=if let Some(t)=&shown{t.clone()}else if c.role=="image"{String::new()}else{c.label.clone()};
        json!({"id":c.id,"bounds":[c.bounds.left,c.bounds.top,c.bounds.right,c.bounds.bottom],"label":label,
            "description":if c.role=="selectable"{""}else{c.label.as_str()},"hint":if c.role=="edit"{c.label.as_str()}else{""},"hintShowing":shown.as_ref().is_some_and(|t| t.is_empty()),"role":c.role,"state":c.summary,"enabled":c.exposed_enabled(),"pressed":false,"selected":c.selected,
            "checked":checked(c),"live":c.role=="status","locked":false,"actions":actions,"longClickLabel":"","holdDelayMillis":500,
            "range":c.range,"anchor":anchor,"focus":focus,"custom":a.custom_actions_for(c.id),"text":a.text_node(c.id)})
    }).collect();
    let field = a
        .controls
        .iter()
        .find(|c| c.id == a.field_id())
        .map(|c| [c.bounds.left, c.bounds.top, c.bounds.right, c.bounds.bottom]);
    let effects = std::mem::take(&mut a.effects);
    let result = json!({"nodes":nodes,"mode":"","shift":"","popup":a.modal.is_some(),"effects":effects,
        "systemBarColor":a.system_bar_color(),"lightSystemBars":a.light_system_bars(),"route":a.stack.last(),"canBack":a.stack.len()>1 || a.modal.is_some() || a.editor.focused,"scroll":a.scroll(),"maxScroll":a.max_scroll,"focus":a.focus,
        "animating":a.animating(),"scrollbarAlpha":a.scrollbar_alpha(usize::from(a.modal.is_some())),"selectionToolbar":a.editor.focused && a.editor.anchor!=a.editor.focus && !a.toolbar_dismissed,"textToolbar":a.text_toolbar(),"accepted":a.accepted,"editor":{"anchor":a.editor.anchor,"focus":a.editor.focus,"composing":a.editor.composing,"focused":a.editor.focused,"report":a.batches.report(),"bounds":field,"multiLine":a.multi_line_field(),"number":a.number_field()}});
    let mut out = vec![vec![]; reply::SIZE];
    out[reply::STATE] = settings::text(&result.to_string());
    out[reply::TEXT] = a.editor.text.clone();
    out[reply::QUERY] = query;
    if saved {
        out.extend(crate::render_settings_state::save(a));
    }
    Ok(out)
}
pub fn settings(native: i64, op: i32, values: &[Vec<u16>]) -> Result<Vec<Vec<u16>>> {
    session(native, |s| {
        let raw = values;
        let p: Vec<_> = raw.iter().map(|v| String::from_utf16_lossy(v)).collect();
        let a = s.settings.get_or_insert_with(App::default);
        a.accepted = true;
        let mut query = vec![];
        match op {
            ops::RESOURCES | ops::PREFERENCES => {
                if p.len() % 2 != 0 {
                    return Err(illegal("settings pairs"));
                }
                for pair in p.as_chunks::<2>().0 {
                    if op == ops::RESOURCES {
                        a.strings.insert(pair[0].clone(), pair[1].clone());
                    } else {
                        a.prefs.insert(pair[0].clone(), pair[1].clone());
                    }
                }
            }
            ops::CONFIG => {
                let c = [float(&p, 0)?, float(&p, 1)?, float(&p, 2)?, float(&p, 3)?];
                if !(1.0..=8192.0).contains(&c[0])
                    || !(1.0..=8192.0).contains(&c[1])
                    || !(0.1..=10.0).contains(&c[2])
                    || !(0.1..=5.0).contains(&c[3])
                {
                    return Err(illegal("settings dimensions"));
                }
                let mut colors = a.colors;
                for (i, color) in colors.iter_mut().take(6).enumerate() {
                    *color = field(&p, 4 + i)?
                        .parse()
                        .map_err(|_| illegal("settings color"))?;
                }
                if p.len() >= 51 {
                    for (i, color) in colors.iter_mut().enumerate().skip(6).take(6) {
                        *color = field(&p, 6 + i)?
                            .parse()
                            .map_err(|_| illegal("settings extended color"))?;
                    }
                    for (i, pixels) in a.sp_pixels.iter_mut().enumerate() {
                        *pixels = float(&p, 18 + i)?;
                    }
                }
                if p.len() > 51 {
                    a.dialog_window_width = float(&p, 51)?;
                }
                if p.len() > 52 {
                    colors[12] = field(&p, 52)?
                        .parse()
                        .map_err(|_| illegal("settings scrolled color"))?;
                }
                if p.len() > 54 {
                    a.vibrator = [field(&p, 53)? == "true", field(&p, 54)? == "true"];
                }
                if p.len() > 55 {
                    colors[13] = field(&p, 55)?
                        .parse()
                        .map_err(|_| illegal("settings card content color"))?;
                }
                // onPrimaryContainer and onBackground, then the time picker's
                // tertiaryContainer and onTertiaryContainer and the text field's error.
                for (i, color) in colors.iter_mut().enumerate().skip(14) {
                    if p.len() > 42 + i {
                        *color = field(&p, 42 + i)?
                            .parse()
                            .map_err(|_| illegal("settings late colors"))?;
                    }
                }
                // 45 sp and 57 sp, the time picker's display sizes, past the sp table.
                if p.len() > 62 {
                    a.display_pixels = [float(&p, 61)?, float(&p, 62)?];
                }
                a.config = c;
                a.colors = colors;
                a.ime_enabled = field(&p, 10)? == "true";
                a.ime_selected = field(&p, 11)? == "true";
            }
            ops::NAVIGATE => a.navigate(field(&p, 0)?),
            ops::ACTIVATE => {
                a.activate(int(&p, 0)?);
            }
            ops::BACK => match int(&p, 0)? {
                back::STARTED | back::PROGRESSED => a.back_progress = float(&p, 1)?.clamp(0.0, 1.0),
                back::CANCELLED => a.back_progress = 0.0,
                back::PRESSED => a.back(),
                _ => return Err(illegal("settings back phase")),
            },
            ops::KEY => {
                // Android's KeyEvent.ACTION_DOWN.
                if int(&p, 0)? == 0 {
                    let key = int(&p, 1)?;
                    let unicode = int(&p, 2)?;
                    if a.editor.focused {
                        // A time field's Enter and step back to the hour come first.
                        if !a.field_key(key, unicode) {
                            match key {
                                67 => a.editor.backspace(),
                                // A multi-line field takes Enter as a new line, finishing any
                                // composition first as Compose's TextFieldKeyInput does.
                                66 if a.field.is_some() => {
                                    a.editor.composing = None;
                                    a.accepted = a.editor.replace(&[10], 1, false);
                                }
                                66 => {
                                    a.editor.focused = false;
                                    a.editor.composing = None;
                                    a.effects.push(vec!["hideEditor".into()]);
                                }
                                21 | 22 => {
                                    let n = (a.editor.focus as i32
                                        + if key == 21 { -1 } else { 1 })
                                    .clamp(0, a.editor.text.len() as i32);
                                    a.editor.select(n, n);
                                }
                                _ => {
                                    if let Some(c) =
                                        char::from_u32(unicode as u32).filter(|c| !c.is_control())
                                    {
                                        a.accepted = a.editor.replace(
                                            &c.to_string().encode_utf16().collect::<Vec<_>>(),
                                            1,
                                            false,
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        match key {
                            19 | 20 | 61 => a.focus_next(key == 19 || field(&p, 3)? == "true"),
                            21 | 22 => {
                                a.adjust(a.focus, if key == 21 { -1.0 } else { 1.0 });
                            }
                            23 | 66 | 62 => {
                                a.activate(a.focus);
                            }
                            _ => {}
                        }
                    }
                }
            }
            ops::FRAME => {
                let now = field(&p, 0)?
                    .parse()
                    .map_err(|_| illegal("settings frame time"))?;
                a.tick(now);
            }
            ops::EDIT => {
                let kind = field(&p, 0)?;
                if !matches!(kind, edit::QUERY | edit::BEGIN | edit::END) {
                    a.toolbar_dismissed = false;
                }
                match kind {
                    edit::COMMIT | edit::COMPOSE => {
                        a.accepted = a.editor.replace(
                            raw.get(1).ok_or_else(|| illegal("settings editor text"))?,
                            int(&p, 2)?,
                            kind == edit::COMPOSE,
                        );
                    }
                    edit::SELECT => {
                        a.accepted = a.editor.select(int(&p, 1)?, int(&p, 2)?);
                    }
                    edit::REGION => a.editor.region(int(&p, 1)?, int(&p, 2)?),
                    edit::FINISH => a.editor.composing = None,
                    edit::BEGIN => {
                        a.batches.begin();
                        a.accepted = true;
                    }
                    edit::END => a.accepted = a.batches.end(),
                    edit::CLOSE => {
                        a.batches.close();
                        a.editor.composing = None;
                    }
                    edit::DELETE | edit::CODEPOINTS => {
                        a.accepted =
                            a.editor
                                .delete(int(&p, 1)?, int(&p, 2)?, kind == edit::CODEPOINTS);
                    }
                    edit::QUERY => {
                        query = if field(&p, 1)? == query::CAPS {
                            settings::text(
                                &editor::caps::flags(
                                    &a.editor.text,
                                    a.editor.focus as i32,
                                    int(&p, 2)?,
                                )
                                .to_string(),
                            )
                        } else {
                            a.editor.query(field(&p, 1)?, int(&p, 2)?.max(0) as usize)
                        };
                    }
                    // An optional control id hands the editor to that field first, as a
                    // tap does, without the effect that shows the IME.
                    edit::FOCUS => {
                        if p.len() > 1 {
                            a.accepted = a.focus_field(int(&p, 1)?);
                        }
                        a.editor.focused = true;
                    }
                    edit::DISMISS_TOOLBAR => a.toolbar_dismissed = true,
                    edit::SELECT_ALL => {
                        a.editor.select(0, a.editor.text.len() as i32);
                    }
                    edit::CUT => {
                        a.editor.replace(&[], 1, false);
                        a.toolbar_dismissed = true;
                    }
                    edit::BLUR => {
                        a.editor.focused = false;
                        a.editor.composing = None;
                    }
                    edit::SET => {
                        a.editor.composing = None;
                        a.editor.select(0, a.editor.text.len() as i32);
                        a.accepted = a.editor.replace(
                            raw.get(1).ok_or_else(|| illegal("settings text"))?,
                            1,
                            false,
                        );
                    }
                    _ => return Err(illegal("settings editor operation")),
                }
            }
            ops::SAVE => {}
            ops::RESTORE => {
                if crate::render_settings_state::restore(a, raw).is_err() {
                    let prefs = std::mem::take(&mut a.prefs);
                    let strings = std::mem::take(&mut a.strings);
                    *a = App {
                        prefs,
                        strings,
                        ..Default::default()
                    };
                }
            }
            ops::CANCEL_TOUCH => {
                a.down = None;
                a.fling = None;
            }
            ops::SCROLL => {
                let delta = float(&p, 0)?;
                a.set_scroll(a.scroll() + delta * a.config[1] * 0.6);
            }
            ops::SLIDER => {
                let id = int(&p, 0)?;
                let v = float(&p, 1)?;
                if let Some(c) = a.controls.iter().find(|c| c.id == id)
                    && let Action::Slider(slot) = c.action
                {
                    a.set_slider(slot, v);
                    // M3's setProgress semantics action ends with onValueChangeFinished.
                    a.slider_finished(slot);
                }
            }
            ops::PLURALS => {
                query = settings::text(&App::plural_requests().join("\n"));
            }
            ops::STRING_KEYS => {
                query = settings::text(&App::string_keys().join("\n"));
            }
            ops::ROUTES => {
                query = settings::text(&catalogue::native_routes().join("\n"));
            }
            ops::FACTS => {
                if p.len() % 2 != 0 {
                    return Err(illegal("settings facts"));
                }
                a.facts = p
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|[k, v]| (k.clone(), v.clone()))
                    .collect();
            }
            ops::DOCUMENTS => {
                // Documents Kotlin read for a `load` plan, as name and text pairs.
                if p.len() % 2 != 0 {
                    return Err(illegal("settings documents"));
                }
                for [name, text] in p.as_chunks::<2>().0 {
                    a.set_document(name, text);
                }
            }
            ops::FILES => a.file_report(&p).map_err(illegal)?,
            ops::LANGUAGES => a.set_languages(&p).map_err(illegal)?,
            ops::WORDS => a.set_words(&p).map_err(illegal)?,
            ops::THEMES => a.set_themes(&p).map_err(illegal)?,
            ops::ASSET => a.set_asset(&p).map_err(illegal)?,
            ops::SELECTION => a.text_selection(&p).map_err(illegal)?,
            ops::ACCESSIBILITY_SCROLL => {
                let id = int(&p, 0)?;
                a.accessibility_scroll(id, int(&p, 1)? == ACCESS_SCROLL_FORWARD);
            }
            _ => return Err(illegal("settings operation")),
        }
        if op == ops::KEY || op == ops::EDIT {
            a.sync_field();
        }
        reply(s, query, op == ops::SAVE)
    })
}
/// The settings picture `name`, `w` by `h` ARGB pixels at its intrinsic size; 0×0 removes it.
pub fn settings_image(
    native: i64,
    name: &[u16],
    w: i32,
    h: i32,
    pixels: &[i32],
) -> Result<Vec<Vec<u16>>> {
    session(native, |s| {
        let a = s.settings.get_or_insert_with(App::default);
        let name = String::from_utf16_lossy(name);
        if w == 0 || h == 0 {
            a.images.remove(&name);
        } else {
            let (Ok(w), Ok(h)) = (u16::try_from(w), u16::try_from(h)) else {
                return Err(illegal("settings image size"));
            };
            let frame = render::media_cache::Frame::argb(w, h, pixels, 0)
                .ok_or_else(|| illegal("settings image pixels"))?;
            a.images.insert(name, frame);
        }
        reply(s, vec![], false)
    })
}
pub fn settings_touch(
    native: i64,
    metadata: &[i32],
    times: &[i64],
    coords: &[f32],
) -> Result<Vec<Vec<u16>>> {
    session(native, |s| {
        let m = metadata;
        if m.len() < 2 {
            return Err(illegal("settings motion metadata"));
        }
        let event = Motion {
            action: m[0],
            index: m[1] as usize,
            ids: m[2..].to_vec(),
            times: times.to_vec(),
            coords: coords.to_vec(),
        };
        s.touch_time = event.times.last().copied().unwrap_or(0);
        s.settings.get_or_insert_with(App::default).motion(&event);
        reply(s, vec![], false)
    })
}
