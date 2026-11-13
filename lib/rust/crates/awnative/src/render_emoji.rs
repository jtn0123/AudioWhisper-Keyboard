//! Android font-support and stored-history data transport for the native emoji palette.
use crate::{
    handle,
    jni::illegal,
    render::{Session, strings},
};
use jni_sys::{JNIEnv, jclass, jintArray, jlong, jobjectArray};
use render::emoji_panel::{EmojiGroup, EmojiPanel, EmojiValue};
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_emoji(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    config: jintArray,
    groups: jintArray,
    values: jobjectArray,
    history: jintArray,
    labels: jobjectArray,
) {
    // SAFETY: JVM synchronized wrapper keeps the session and input arrays live; all data is copied.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let c = env.read::<i32>(config, "emoji settings")?;
            let g = env.read::<i32>(groups, "emoji groups")?;
            let v = strings(env, values)?;
            let h = env.read::<i32>(history, "emoji history")?;
            let labels = strings(env, labels)?;
            if c.len() < 8
                || c.len() != 8 + v.len()
                || g.len() % 3 != 0
                || h.is_empty()
                || h[0] < 0
                || h.len() < 1 + h[0] as usize
                || labels.len() != 29
            {
                return Err(illegal("emoji payload"));
            }
            let entries: Vec<_> = v
                .into_iter()
                .enumerate()
                .map(|(i, value)| EmojiValue {
                    value,
                    tone: c[8 + i],
                })
                .collect();
            let groups: Vec<_> = g
                .as_chunks::<3>()
                .0
                .iter()
                .map(|a| EmojiGroup {
                    category: a[0],
                    start: a[1].max(0) as usize,
                    count: a[2].max(0) as usize,
                })
                .collect();
            if groups.iter().any(|g| g.start + g.count > entries.len())
                || h[1..]
                    .iter()
                    .any(|i| *i < 0 || *i as usize >= entries.len())
            {
                return Err(illegal("emoji indices"));
            }
            let Some(p) = &mut s.scene.panel else {
                return Ok(());
            };
            let pinned = h[1..1 + h[0] as usize]
                .iter()
                .map(|i| *i as usize)
                .collect();
            let recent = h[1 + h[0] as usize..].iter().map(|i| *i as usize).collect();
            if let Some(e) = &mut p.emoji {
                let reset = e.history != (c[0] != 0);
                e.history = c[0] != 0;
                e.locked = c[1] != 0;
                e.tone = c[2];
                e.pinned = pinned;
                e.recent = recent;
                e.values = entries;
                e.groups = groups;
                e.gifs = c[6] != 0;
                e.bottom = c[5] as f32;
                if reset {
                    e.category = if e.history { 0 } else { 1 };
                    e.refresh();
                    p.scroll = 0.0;
                }
                e.pinned_strategy = c[3];
                e.recent_strategy = c[4];
                e.labels = labels[..15].to_vec();
                e.icons = labels[15..].to_vec();
                if c[7] != 0 {
                    e.refresh();
                }
            } else {
                let mut e = EmojiPanel {
                    values: entries,
                    groups,
                    pinned,
                    recent,
                    shown_pinned: vec![],
                    shown_recent: vec![],
                    history: c[0] != 0,
                    locked: c[1] != 0,
                    tone: c[2],
                    pinned_strategy: c[3],
                    recent_strategy: c[4],
                    category: if c[0] != 0 { 0 } else { 1 },
                    popup: None,
                    labels: labels[..15].to_vec(),
                    icons: labels[15..].to_vec(),
                    bottom: c[5] as f32,
                    gifs: c[6] != 0,
                };
                e.refresh();
                p.emoji = Some(e);
            }
            s.prepare_panel();
            Ok(())
        });
    }
}
