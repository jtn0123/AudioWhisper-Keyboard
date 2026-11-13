//! Clipboard data transport. Android retains Room objects; native retains display snapshots only.
use crate::{
    handle,
    jni::illegal,
    render::{Session, strings},
};
use jni_sys::{JNIEnv, jclass, jlong, jlongArray, jobjectArray};
use render::clipboard_panel::{ClipEntry, ClipboardPanel};
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_clipboard(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    config: jlongArray,
    records: jlongArray,
    labels: jobjectArray,
) {
    // SAFETY: synchronized JVM owner retains the session; inputs are copied before any update.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let c = env.read::<i64>(config, "clipboard config")?;
            let a = env.read::<i64>(records, "clipboard entries")?;
            let labels = strings(env, labels)?;
            if c.len() != 4 || a.len() % 4 != 0 || labels.len() != 31 + a.len() / 4 * 3 {
                return Err(illegal("clipboard snapshot"));
            }
            let next = ClipboardPanel {
                now: c[0],
                enabled: c[1] != 0,
                locked: c[2] != 0,
                columns: c[3].max(0) as usize,
                labels: labels[..21].to_vec(),
                icons: labels[21..31].to_vec(),
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
                        label: labels[31 + i * 3].clone(),
                        description: labels[32 + i * 3].clone(),
                        date: labels[33 + i * 3].clone(),
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
        });
    }
}
