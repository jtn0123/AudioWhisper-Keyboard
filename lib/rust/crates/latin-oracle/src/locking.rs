//! The lock around the C++.
//!
//! Every call into the C++ takes one lock, shared ([`lock`]), so the tests can run the C++ on
//! several threads at once; only the calls that write a global take it exclusively
//! ([`exclusive_lock`]): JNI_OnLoad, and the header hook's `HeaderPolicy`, which sets
//! `TimeKeeper`. Sharing is safe because nothing else a call reaches writes anything but objects
//! its caller owns:
//!
//! * no class in `lib/latinime` has a `mutable` member, the only non-`const` statics are
//!   `TimeKeeper`'s and two debug-only dump buffers (compiled out), and the typing policies'
//!   singletons are `const`;
//! * a search (`Dictionary::getSuggestions`, which is `const`) writes its session and its
//!   arrays and reads the dictionary and the `ProximityInfo`; [`Session`] has a lock of its own,
//!   so two searches never share a session;
//! * a session's constructor, `init` and destructor touch only that session and read the
//!   dictionary; the other hooks read a dictionary or keyboard, or write a `ProximityInfoState`
//!   the caller owns (a hook on a session's state must not run while that session searches);
//! * the fake JNI environment's natives are written only by JNI_OnLoad; its classes are also
//!   added after it, by any call whose C++ calls FindClass (every dictionary open logs, which
//!   looks up android.util.Log), so that table has a mutex of its own; its arrays are each call's
//!   own;
//! * `TimeKeeper::setCurrentTime`, which every search calls, writes nothing in test mode, and
//!   `TimeKeeper` is held in test mode whenever the lock isn't held exclusively. Only `HeaderPolicy`
//!   reads the time, for a missing date, and only the header hook (exclusive) controls it.

use std::sync::{MutexGuard, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{Session, aw_oracle_hold_time, aw_oracle_load, aw_oracle_release_time};

/// The lock every call into the C++ holds: shared ([`lock`]), or exclusively by the calls that
/// write a global ([`exclusive_lock`]).
static LOCK: RwLock<()> = RwLock::new(());

/// [`LOCK`] held exclusively: no other call into the C++ is running. While it is held,
/// `TimeKeeper` is on the clock, as a lone call finds it; outside it, it is held still.
#[derive(Debug)]
pub struct Exclusive {
    _guard: RwLockWriteGuard<'static, ()>,
}

impl Drop for Exclusive {
    fn drop(&mut self) {
        // SAFETY: only TimeKeeper's statics change, and no other call is running.
        unsafe { aw_oracle_hold_time() };
    }
}

/// The lock for a call that writes one of the C++'s globals: JNI_OnLoad (the fake environment's
/// tables of natives and classes) and `HeaderPolicy`'s constructor as the header hook makes it
/// (`TimeKeeper`). Taken after JNI_OnLoad has registered the natives.
pub fn exclusive_lock() -> Exclusive {
    static LOADED: OnceLock<i32> = OnceLock::new();
    let guard = LOCK.write().unwrap_or_else(|it| it.into_inner());
    // SAFETY: JNI_OnLoad only registers natives into the fake environment; it runs once, locked.
    let version = *LOADED.get_or_init(|| unsafe { aw_oracle_load() });
    assert!(version > 0, "LatinIME's JNI_OnLoad failed");
    // SAFETY: only TimeKeeper's statics change, and no other call is running.
    unsafe { aw_oracle_release_time() };
    Exclusive { _guard: guard }
}

/// The lock every other call into the C++ holds, shared: such a call writes only objects its
/// caller owns (a session, a `ProximityInfoState`, its own arrays) and reads the rest, so several
/// run at once. `TimeKeeper` is held while it is, so `Dictionary::getSuggestions`'s
/// `setCurrentTime` writes nothing.
pub fn lock() -> RwLockReadGuard<'static, ()> {
    static STARTED: OnceLock<()> = OnceLock::new();
    // JNI_OnLoad first, once, and TimeKeeper held, as every exclusive hold leaves it.
    STARTED.get_or_init(|| drop(exclusive_lock()));
    LOCK.read().unwrap_or_else(|it| it.into_inner())
}

/// What a search holds: `session`'s own lock, then [`lock`]. Searches on different sessions run
/// at once; one on the same session waits.
pub(crate) fn search_lock(session: &Session) -> (MutexGuard<'_, ()>, RwLockReadGuard<'static, ()>) {
    let own = session.busy.lock().unwrap_or_else(|it| it.into_inner());
    (own, lock())
}
