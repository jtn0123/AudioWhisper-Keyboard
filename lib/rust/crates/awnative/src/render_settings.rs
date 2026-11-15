//! JNI copies only: settings policy/geometry/editor live in the unsafe-free renderer module.
use crate::{
    handle,
    jni::{Result, illegal},
    render::Session,
};
use jni_sys::{JNIEnv, jclass, jfloatArray, jint, jintArray, jlong, jlongArray, jobjectArray};
use render::{
    settings_ui::{Action, App, catalogue},
    touch::Motion,
};
use serde_json::json;
use std::ptr;
fn input(env: &crate::jni::Env, values: jobjectArray) -> Result<Vec<Vec<u16>>> {
    env.read_strings(values, "settings input")?
        .into_iter()
        .map(|v| v.ok_or_else(|| illegal("null settings field")))
        .collect()
}
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
fn reply(
    s: &mut Session,
    env: &crate::jni::Env,
    query: Vec<u16>,
    saved: bool,
) -> Result<jobjectArray> {
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
    let nodes:Vec<_>=a.controls.iter().map(|c|{
        let mut actions=if !c.enabled || matches!(c.action,Action::None){vec![]}else{vec![1]};
        if c.enabled && c.role=="slider" {actions.extend([4,5]);} else if c.role=="scroll" {let [back,forward]=a.can_scroll(); if forward {actions.push(4);}
            if back {actions.push(5);}}
        // An image has a content description and no text, as Compose's Image does.
        let edited=(c.role=="edit").then(|| a.field_editor(c.id).0);
        let shown=edited.as_ref().map(|e| String::from_utf16_lossy(&e.text));
        let (anchor,focus)=edited.as_ref().map_or((a.editor.anchor,a.editor.focus),|e| (e.anchor,e.focus));
        let label=if let Some(t)=&shown{t.clone()}else if c.role=="image"{String::new()}else{c.label.clone()};
        json!({"id":c.id,"bounds":[c.bounds.left,c.bounds.top,c.bounds.right,c.bounds.bottom],"label":label,
            "description":c.label,"hint":if c.role=="edit"{c.label.as_str()}else{""},"hintShowing":shown.as_ref().is_some_and(|t| t.is_empty()),"role":c.role,"state":c.summary,"enabled":c.enabled,"pressed":false,"selected":c.selected,
            "checked":if c.role=="radio"{Some(c.selected)}else{c.checked},"locked":false,"actions":actions,"longClickLabel":"","holdDelayMillis":500,
            "range":c.range,"anchor":anchor,"focus":focus})
    }).collect();
    let field = a
        .controls
        .iter()
        .find(|c| c.id == a.field_id())
        .map(|c| [c.bounds.left, c.bounds.top, c.bounds.right, c.bounds.bottom]);
    let effects = std::mem::take(&mut a.effects);
    let result = json!({"nodes":nodes,"mode":"","shift":"","popup":a.modal.is_some(),"effects":effects,
        "systemBarColor":a.system_bar_color(),"lightSystemBars":a.light_system_bars(),"route":a.stack.last(),"canBack":a.stack.len()>1 || a.modal.is_some() || a.editor.focused,"scroll":a.scroll(),"maxScroll":a.max_scroll,"focus":a.focus,
        "animating":a.animating(),"scrollbarAlpha":a.scrollbar_alpha(usize::from(a.modal.is_some())),"selectionToolbar":a.editor.focused && a.editor.anchor!=a.editor.focus && !a.toolbar_dismissed,"accepted":a.accepted,"editor":{"anchor":a.editor.anchor,"focus":a.editor.focus,"composing":a.editor.composing,"focused":a.editor.focused,"bounds":field,"multiLine":a.field.is_some()}});
    let mut out = vec![
        settings::text(&result.to_string()),
        a.editor.text.clone(),
        query,
    ];
    if saved {
        out.extend(crate::render_settings_state::save(a));
    }
    env.make_strings(&out)
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_settings(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    op: jint,
    values: jobjectArray,
) -> jobjectArray {
    // SAFETY: synchronized JVM owner retains the session; all JNI input/output references are copied local arrays.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            let raw = input(env, values)?;
            let p: Vec<_> = raw.iter().map(|v| String::from_utf16_lossy(v)).collect();
            let a = s.settings.get_or_insert_with(App::default);
            a.accepted = true;
            let mut query = vec![];
            match op {
                1 | 2 => {
                    if p.len() % 2 != 0 {
                        return Err(illegal("settings pairs"));
                    }
                    for pair in p.as_chunks::<2>().0 {
                        if op == 1 {
                            a.strings.insert(pair[0].clone(), pair[1].clone());
                        } else {
                            a.prefs.insert(pair[0].clone(), pair[1].clone());
                        }
                    }
                }
                3 => {
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
                    a.config = c;
                    a.colors = colors;
                    a.ime_enabled = field(&p, 10)? == "true";
                    a.ime_selected = field(&p, 11)? == "true";
                }
                4 => a.navigate(field(&p, 0)?),
                5 => {
                    a.activate(int(&p, 0)?);
                }
                6 => match int(&p, 0)? {
                    0 | 1 => a.back_progress = float(&p, 1)?.clamp(0.0, 1.0),
                    2 => a.back_progress = 0.0,
                    3 => a.back(),
                    _ => return Err(illegal("settings back phase")),
                },
                7 => {
                    if int(&p, 0)? == 0 {
                        let key = int(&p, 1)?;
                        let unicode = int(&p, 2)?;
                        if a.editor.focused {
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
                8 => {
                    let now = field(&p, 0)?
                        .parse()
                        .map_err(|_| illegal("settings frame time"))?;
                    a.tick(now);
                }
                9 => {
                    let kind = field(&p, 0)?;
                    if kind != "query" {
                        a.toolbar_dismissed = false;
                    }
                    match kind {
                        "commit" | "compose" => {
                            a.accepted = a.editor.replace(
                                raw.get(1).ok_or_else(|| illegal("settings editor text"))?,
                                int(&p, 2)?,
                                kind == "compose",
                            );
                        }
                        "select" => {
                            a.accepted = a.editor.select(int(&p, 1)?, int(&p, 2)?);
                        }
                        "region" => a.editor.region(int(&p, 1)?, int(&p, 2)?),
                        "finish" => a.editor.composing = None,
                        "delete" | "codepoints" => {
                            a.accepted =
                                a.editor
                                    .delete(int(&p, 1)?, int(&p, 2)?, kind == "codepoints");
                        }
                        "query" => {
                            query = if field(&p, 1)? == "caps" {
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
                        "focus" => {
                            if p.len() > 1 {
                                a.accepted = a.focus_field(int(&p, 1)?);
                            }
                            a.editor.focused = true;
                        }
                        "dismissToolbar" => a.toolbar_dismissed = true,
                        "selectAll" => {
                            a.editor.select(0, a.editor.text.len() as i32);
                        }
                        "cut" => {
                            a.editor.replace(&[], 1, false);
                            a.toolbar_dismissed = true;
                        }
                        "blur" => {
                            a.editor.focused = false;
                            a.editor.composing = None;
                        }
                        "set" => {
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
                10 => {}
                11 => {
                    if crate::render_settings_state::restore(a, &raw).is_err() {
                        let prefs = std::mem::take(&mut a.prefs);
                        let strings = std::mem::take(&mut a.strings);
                        *a = App {
                            prefs,
                            strings,
                            ..Default::default()
                        };
                    }
                }
                12 => {
                    a.down = None;
                    a.fling = None;
                }
                14 => {
                    let delta = float(&p, 0)?;
                    a.set_scroll(a.scroll() + delta * a.config[1] * 0.6);
                }
                15 => {
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
                16 => {
                    query = settings::text(&App::plural_requests().join("\n"));
                }
                18 => {
                    query = settings::text(&App::string_keys().join("\n"));
                }
                19 => {
                    query = settings::text(&catalogue::native_routes().join("\n"));
                }
                20 => {
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
                17 => {
                    let id = int(&p, 0)?;
                    let direction = if int(&p, 1)? == 4 { 1.0 } else { -1.0 };
                    if !a.adjust(id, direction) {
                        a.set_scroll(a.scroll() + direction * a.config[1] * 0.6);
                    }
                }
                _ => return Err(illegal("settings operation")),
            }
            if op == 7 || op == 9 {
                a.sync_field();
            }
            reply(s, env, query, op == 10)
        })
    }
}
/// A picture for the settings screens (`values` = name, width, height) as ARGB pixels at its
/// intrinsic size; an empty one removes it.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_settingsImage(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    values: jobjectArray,
    pixels: jintArray,
) -> jobjectArray {
    // SAFETY: the synchronized wrapper retains the live handle; the name and pixel arrays are
    // copied before use.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            let raw = input(env, values)?;
            let p: Vec<_> = raw.iter().map(|v| String::from_utf16_lossy(v)).collect();
            let (w, h) = (int(&p, 1)?, int(&p, 2)?);
            let pixels = env.read::<i32>(pixels, "settings image")?;
            let a = s.settings.get_or_insert_with(App::default);
            let name = field(&p, 0)?.to_owned();
            if w == 0 || h == 0 {
                a.images.remove(&name);
            } else {
                let (Ok(w), Ok(h)) = (u16::try_from(w), u16::try_from(h)) else {
                    return Err(illegal("settings image size"));
                };
                let frame = render::media_cache::Frame::argb(w, h, &pixels, 0)
                    .ok_or_else(|| illegal("settings image pixels"))?;
                a.images.insert(name, frame);
            }
            reply(s, env, vec![], false)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_settingsTouch(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    metadata: jintArray,
    times: jlongArray,
    coords: jfloatArray,
) -> jobjectArray {
    // SAFETY: the synchronized wrapper retains the live handle; MotionEvent arrays are copied before use.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            let m = env.read::<i32>(metadata, "settings motion")?;
            if m.len() < 2 {
                return Err(illegal("settings motion metadata"));
            }
            let event = Motion {
                action: m[0],
                index: m[1] as usize,
                ids: m[2..].to_vec(),
                times: env.read(times, "settings times")?,
                coords: env.read(coords, "settings coordinates")?,
            };
            s.touch_time = event.times.last().copied().unwrap_or(0);
            s.settings.get_or_insert_with(App::default).motion(&event);
            reply(s, env, vec![], false)
        })
    }
}
