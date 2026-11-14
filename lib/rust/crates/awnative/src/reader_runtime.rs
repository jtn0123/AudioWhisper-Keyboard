//! One native walk with Android callbacks; no policy or native locks cross the callbacks.
use crate::{
    handle,
    jni::{self, Env, Method, illegal},
};
use jni_sys::{JNIEnv, jclass, jint, jintArray, jlong, jlongArray, jobject, jvalue};
use reader::{
    play_queue::PlayQueue,
    screen::Bounds,
    walk::{self, Node, Tree, Walk},
};
use std::{ptr, sync::Mutex};
struct AndroidTree<'a> {
    e: &'a Env,
    facts: Method,
    class: Method,
    text: Method,
    child: Method,
}
impl Tree for AndroidTree<'_> {
    type Error = jni::Error;
    fn node(&mut self, id: i32) -> jni::Result<Node> {
        let raw = self.e.call_object(self.facts, &[jvalue { i: id }])?;
        let fields = self.e.read::<i32>(raw, "node facts");
        self.e.delete_local(raw);
        let f = fields?;
        if f.len() != 13 {
            return Err(illegal("Node facts length"));
        }
        let raw = self.e.call_object(self.class, &[jvalue { i: id }])?;
        let class = self.e.read_string(raw, "node class");
        self.e.delete_local(raw);
        Ok(Node {
            bounds: Bounds {
                left: f[0],
                top: f[1],
                right: f[2],
                bottom: f[3],
            },
            password: f[4] != 0,
            editable: f[5] != 0,
            visible: f[6] != 0,
            scrollable: f[7] != 0,
            rows: f[8],
            columns: f[9],
            up_down: f[10] != 0,
            across: f[11] != 0,
            children: f[12],
            class: class?,
        })
    }
    fn text(&mut self, id: i32) -> jni::Result<Vec<u16>> {
        let raw = self.e.call_object(self.text, &[jvalue { i: id }])?;
        let result = self.e.read_string(raw, "node text");
        self.e.delete_local(raw);
        result
    }
    fn child(&mut self, id: i32, index: i32) -> jni::Result<Option<i32>> {
        let value = self
            .e
            .call_int(self.child, &[jvalue { i: id }, jvalue { i: index }])?;
        Ok((value >= 0).then_some(value))
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderWalk_walk(
    env: *mut JNIEnv,
    _: jclass,
    tree: jobject,
    visible: jintArray,
) -> jintArray {
    // SAFETY: all callbacks use this call's thread/local references. No native lock is held.
    unsafe {
        jni::run(env, ptr::null_mut(), |e| {
            let v = e.read::<i32>(visible, "visible bounds")?;
            if v.len() != 4 {
                return Err(illegal("Visible bounds length"));
            }
            let mut tree = AndroidTree {
                e,
                facts: e.method(tree, "tree", (c"facts", c"(I)[I"))?,
                class: e.method(tree, "tree", (c"className", c"(I)Ljava/lang/String;"))?,
                text: e.method(tree, "tree", (c"text", c"(I)Ljava/lang/String;"))?,
                child: e.method(tree, "tree", (c"child", c"(II)I"))?,
            };
            e.make(
                &Walk::run(
                    &mut tree,
                    Bounds {
                        left: v[0],
                        top: v[1],
                        right: v[2],
                        bottom: v[3],
                    },
                )?
                .packet(),
            )
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ReaderWalk_followConfig(
    env: *mut JNIEnv,
    _: jclass,
) -> jlongArray {
    // SAFETY: JVM environment for the returned owned array.
    unsafe {
        jni::run(env, ptr::null_mut(), |e| {
            e.make(&[
                i64::from(walk::MAX_TURNS),
                walk::TURN_MS,
                reader::play_queue::LOOKAHEAD,
                reader::play_queue::FOLLOW_MS,
            ])
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PlaybackNative_create(
    _: *mut JNIEnv,
    _: jclass,
) -> jlong {
    handle::into_handle(Mutex::new(PlayQueue::default()))
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PlaybackNative_free(
    _: *mut JNIEnv,
    _: jclass,
    h: jlong,
) {
    // SAFETY: synchronized managed allocation frees once after all calls finish.
    unsafe { handle::free::<Mutex<PlayQueue>>(h) }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PlaybackNative_call(
    env: *mut JNIEnv,
    _: jclass,
    h: jlong,
    op: jint,
    id: jint,
    frame: jlong,
) -> jlongArray {
    // SAFETY: synchronized wrapper owns this queue; no callbacks while its lock is held.
    unsafe {
        handle::with_locked::<PlayQueue, _>(env, h, "playback queue", ptr::null_mut(), |q, e| {
            e.make(&q.call(op, id, frame).map_err(illegal)?)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PlaybackNative_system(
    env: *mut JNIEnv,
    _: jclass,
    h: jlong,
    op: jint,
    id: jint,
    text: jni_sys::jstring,
) -> jlongArray {
    // SAFETY: synchronized wrapper owns the queue; UTF-16 is copied and no callbacks run locked.
    unsafe {
        handle::with_locked::<PlayQueue, _>(env, h, "speech queue", ptr::null_mut(), |q, e| {
            e.make(
                &q.system(op, id, &e.read_string(text, "utterance id")?)
                    .map_err(illegal)?,
            )
        })
    }
}
