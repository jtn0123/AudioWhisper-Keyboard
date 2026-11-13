//! Pure policy probes used by JVM differential tests; production uses the same functions.
use crate::jni::{self, illegal};
use jni_sys::{JNIEnv, jclass, jfloatArray, jintArray, jlong};
use std::ptr;
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_editing(
    env: *mut JNIEnv,
    _: jclass,
    codes: jintArray,
    hold: jlong,
    repeat: jlong,
    words: jlong,
) -> jintArray {
    // SAFETY: JVM owns local arrays for the call; every output is copied into a local array.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let codes = env.read::<i32>(codes, "codes")?;
            let out: Vec<_> = codes
                .iter()
                .flat_map(|&code| {
                    let r = render::editing::repeat_code(code, words != 0);
                    [
                        i32::from(render::editing::repeatable(code)),
                        r,
                        render::editing::hold_delay(code, hold) as i32,
                        render::editing::repeat_delay(r, repeat) as i32,
                        i32::from(render::editing::closes(code)),
                    ]
                })
                .collect();
            env.make(&out)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_cells(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
) -> jfloatArray {
    // SAFETY: local arrays are copied; the returned reference belongs to the JVM.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<f32>(config, "grid")?;
            if c.len() != 2 || c.iter().any(|v| !v.is_finite() || *v <= 0.0) {
                return Err(illegal("grid config"));
            }
            env.make(
                &render::panel_geometry::cells(c[0], c[1])
                    .iter()
                    .flat_map(|(x, w)| [*x, *w])
                    .collect::<Vec<_>>(),
            )
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_emojiChoices(
    env: *mut JNIEnv,
    _: jclass,
    tones: jintArray,
    tone: i32,
) -> jintArray {
    // SAFETY: JVM supplies the local primitive array; no native object or borrowed memory escapes.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let tones = env.read::<i32>(tones, "tones")?;
            let mut out = vec![render::panel_geometry::base(&tones, tone) as i32];
            out.extend(
                render::panel_geometry::variants(&tones, tone)
                    .iter()
                    .map(|i| *i as i32),
            );
            env.make(&out)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_selection(
    env: *mut JNIEnv,
    _: jclass,
    config: jintArray,
) -> jintArray {
    // SAFETY: JVM owns the local arrays; Rust returns a copied editor-operation plan.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c: [i32; 6] = env
                .read::<i32>(config, "selection")?
                .try_into()
                .map_err(|_| illegal("selection config"))?;
            env.make(&render::editing::selection(c))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_scroll(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
) -> jfloatArray {
    // SAFETY: JVM owns the input array and receives a copied primitive result.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c: [f32; 3] = env
                .read::<f32>(config, "scroll")?
                .try_into()
                .map_err(|_| illegal("scroll config"))?;
            let (duration, distance) = render::panel_scroll::decay(c[0], c[1]);
            env.make(&[
                duration as f32,
                distance,
                distance * c[1].signum() * render::panel_scroll::fraction(c[2]),
            ])
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_overflowAndHistoryMoves(
    env: *mut JNIEnv,
    _: jclass,
    config: jintArray,
) -> jintArray {
    // SAFETY: the JVM owns local primitive arrays; only a copied result is returned.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c: [i32; 5] = env
                .read::<i32>(config, "panel policy")?
                .try_into()
                .map_err(|_| illegal("panel policy config"))?;
            let arrangement = render::arrangement::Arrangement {
                dynamic: (0..c[0].max(0)).collect(),
                ..Default::default()
            };
            env.make(&[
                arrangement.overflow(c[1].max(0) as usize).len() as i32,
                i32::from(render::panel_geometry::history_moves(c[2] != 0, c[3], c[4])),
            ])
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_clipboardGrid(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
) -> jfloatArray {
    // SAFETY: JVM arrays are copied and the pure grid probe returns a fresh local primitive array.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<f32>(config, "clipboard grid")?;
            if c.len() < 3 || c.iter().any(|v| !v.is_finite()) || c[0] <= 0.0 || c[1] <= 0.0 {
                return Err(illegal("clipboard grid"));
            }
            let n = render::clipboard_panel::columns(c[0], c[1], c[2].max(0.0) as usize);
            let mut lanes = vec![0.0; n];
            let mut out = vec![n as f32];
            for &height in &c[3..] {
                let lane = render::clipboard_panel::lane(&lanes);
                out.extend([lane as f32, lanes[lane]]);
                lanes[lane] += height;
            }
            env.make(&out)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_clipboardMenu(
    env: *mut JNIEnv,
    _: jclass,
    config: jintArray,
) -> jintArray {
    // SAFETY: input/output are local JVM arrays; the probe uses owned synthetic entries only.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<i32>(config, "clipboard menu")?;
            if c.len() != 8 {
                return Err(illegal("clipboard menu"));
            }
            let mut p = render::panels::Panel {
                scroll: 50.0,
                clipboard: Some(render::clipboard_panel::ClipboardPanel {
                    enabled: c[0] != 0,
                    locked: c[1] != 0,
                    filter_row: c[2] != 0,
                    filters: c[3] as u32,
                    popup: (c[4] >= 0).then_some(c[4] as usize),
                    clear: c[5] != 0,
                    entries: (0..4)
                        .map(|i| render::clipboard_panel::ClipEntry {
                            id: i,
                            semantic: i as i32,
                            timestamp: 0,
                            kind: (i % 3 + 1) as i32,
                            pinned: i == 0,
                            label: String::new(),
                            description: String::new(),
                            date: String::new(),
                            aspect: 1.0,
                            height: 1.0,
                            date_height: 1.0,
                            full_height: 1.0,
                        })
                        .collect(),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let actions = p.clipboard_activate(c[6], c[7] as usize);
            let state = p.clipboard.unwrap();
            let mut out = vec![
                i32::from(state.filter_row),
                state.filters as i32,
                state.popup.map_or(-1, |i| i as i32),
                i32::from(state.clear),
                p.scroll as i32,
            ];
            out.extend(actions.into_iter().flat_map(|a| [a.0, a.1, a.2]));
            env.make(&out)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_clipboardTargets(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
) -> jfloatArray {
    // SAFETY: JVM arrays are copied; returned target rectangles contain only synthetic geometry.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<f32>(config, "clipboard targets")?;
            if c.len() != 14 || c.iter().any(|v| !v.is_finite()) || c[0] <= 0.0 {
                return Err(illegal("clipboard targets"));
            }
            let mut p = render::panels::Panel {
                density: c[0],
                ..Default::default()
            };
            for i in 0..2 {
                let at = 2 + i * 4;
                let mut item = render::panels::Item::new(
                    i as i32,
                    render::panels::rect(c[at], c[at + 1], c[at + 2], c[at + 3]),
                    String::new(),
                    if c[1] == 0.0 {
                        "clipboard-item"
                    } else {
                        "clipboard-header-button"
                    },
                    render::panels::Command::Clipboard(5, i),
                );
                item.clip = keys::Rect {
                    left: c[10],
                    top: c[11],
                    right: c[12],
                    bottom: c[13],
                };
                p.items.push(item);
            }
            let r = p.target_bounds(0);
            env.make(&[r.left, r.top, r.right, r.bottom])
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_clipboardDialog(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
) -> jfloatArray {
    // SAFETY: JVM local geometry arrays are copied; output is a new local array.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<f32>(config, "clipboard dialog")?;
            if c.len() != 18 || c.iter().any(|v| !v.is_finite()) {
                return Err(illegal("clipboard dialog"));
            }
            let m = render::clipboard_dialog::ClearMetrics {
                root: c[5..9].try_into().unwrap(),
                row: c[9..13].try_into().unwrap(),
                messages: [c[13]; 2],
                buttons: [[c[14], c[15]], [c[16], c[17]]],
            };
            let rects = render::clipboard_dialog::geometry(
                c[0],
                render::panels::rect(c[1], c[2], c[3], c[4]),
                &m,
                false,
            );
            env.make(
                &rects
                    .into_iter()
                    .flat_map(|r| [r.left, r.top, r.right, r.bottom])
                    .collect::<Vec<_>>(),
            )
        })
    }
}
