//! GIF results metadata; network and decoding remain at the Android edge.
use crate::{
    handle,
    jni::illegal,
    render::{Session, strings},
};
use jni_sys::{JNIEnv, jfloatArray, jint, jlong, jlongArray, jobject, jobjectArray};
use render::gif_panel::{GifEntry, GifPanel};
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_gifs(
    env: *mut JNIEnv,
    _: jobject,
    native: jlong,
    config: jfloatArray,
    labels: jobjectArray,
    chips: jint,
    ids: jlongArray,
) {
    // SAFETY: synchronized owner retains the session; copied metadata has no Android references.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let ids = env.read::<i64>(ids, "gif identities")?;
            let c = env.read::<f32>(config, "gif config")?;
            let labels = strings(env, labels)?;
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
        });
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_gifSent(
    env: *mut JNIEnv,
    _: jobject,
    native: jlong,
) {
    // SAFETY: synchronized wrapper owns the session for the completion notification.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, _| {
            if let Some(p) = &mut s.scene.panel
                && let Some(g) = &mut p.gif
            {
                g.sending = None;
            }
            s.prepare_panel();
            Ok(())
        });
    }
}
