//! The reader's screen walk, with Android callbacks, and its play queue, for `ReaderWalk` and
//! `PlaybackNative`, whose generated entry points call these. No policy or native lock crosses
//! the callbacks.

use std::sync::{Mutex, MutexGuard};

use jni_sys::{jobject, jvalue};
use reader::{
    play_queue::PlayQueue,
    screen::Bounds,
    walk::{self, Node, Tree, Walk},
};

use crate::handle::{into_handle, locked};
use crate::jni::{self, Env, Error, Method, Result, illegal};

/// The Kotlin `ReaderTree` the walk asks, with its methods looked up once.
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
            bounds: bounds(&f),
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

/// Bounds from their first four numbers: left, top, right, bottom.
fn bounds(v: &[i32]) -> Bounds {
    Bounds {
        left: v[0],
        top: v[1],
        right: v[2],
        bottom: v[3],
    }
}

/// What to read of `tree` within the `visible` bounds, as the walk's packet. The callbacks run
/// on this call's thread with its local references, and no native lock is held.
pub fn walk(env: &Env, tree: jobject, visible: &[i32]) -> Result<Vec<i32>> {
    if visible.len() != 4 {
        return Err(illegal("Visible bounds length"));
    }
    let mut tree = AndroidTree {
        e: env,
        facts: env.method(tree, "tree", (c"facts", c"(I)[I"))?,
        class: env.method(tree, "tree", (c"className", c"(I)Ljava/lang/String;"))?,
        text: env.method(tree, "tree", (c"text", c"(I)Ljava/lang/String;"))?,
        child: env.method(tree, "tree", (c"child", c"(II)I"))?,
    };
    Ok(Walk::run(&mut tree, bounds(visible))?.packet())
}

/// The most page turns, the wait for each, the play queue's lookahead and the follow-along delay.
pub fn follow_config() -> Vec<i64> {
    vec![
        i64::from(walk::MAX_TURNS),
        walk::TURN_MS,
        reader::play_queue::LOOKAHEAD,
        reader::play_queue::FOLLOW_MS,
    ]
}

/// A new, empty play queue.
pub fn create() -> i64 {
    into_handle(Mutex::new(PlayQueue::default()))
}

pub fn free(handle: i64) -> Result<()> {
    // SAFETY: the wrapper serializes close with calls, zeros its handle and frees exactly once.
    unsafe { crate::handle::free::<Mutex<PlayQueue>>(handle) };
    Ok(())
}

/// The queue behind `handle`, locked. A closed queue throws an `IllegalStateException`, as
/// Kotlin's `check` did.
fn queue<'a>(handle: i64) -> Result<MutexGuard<'a, PlayQueue>> {
    if handle == 0 {
        return Err(Error::IllegalState("Playback queue closed".into()));
    }
    // SAFETY: the synchronized wrapper keeps the handle live and serializes close.
    unsafe { locked(handle, "playback queue") }
}

/// The queue's answer to `op` for utterance `id` at `frame`.
pub fn call(handle: i64, op: i32, id: i32, frame: i64) -> Result<Vec<i64>> {
    queue(handle)?.call(op, id, frame).map_err(illegal)
}

/// The queue's answer to the system voice's `op` for utterance `id`, whose system id is `text`.
pub fn system(handle: i64, op: i32, id: i32, text: &[u16]) -> Result<Vec<i64>> {
    queue(handle)?.system(op, id, text).map_err(illegal)
}
