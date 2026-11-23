//! The clipboard history and its media files' records, and every change made to them.
//!
//! The history is kept as Room kept it: rows in the order of their ids, which only grow, and
//! shown newest first by [`crate::clipboard_policy::order`]. Each change applies the history's
//! size limit after it, as the keyboard did each time Room reported a change. A change that
//! removes the user's media clips reports the media no clip refers to any more, for the keyboard
//! to delete; the limit and expiry remove rows only, as before, since the system clipboard may
//! still hold one of theirs.

use crate::clipboard_policy as policy;

/// A clip's type, as `ItemType` numbers them.
pub const TEXT: i32 = 1;
pub const IMAGE: i32 = 2;
pub const VIDEO: i32 = 3;

/// One clip of the history.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Clip {
    pub id: i64,
    pub kind: i32,
    /// When it was copied, in milliseconds since 1970.
    pub created: i64,
    pub pinned: bool,
    pub sensitive: bool,
    pub remote: bool,
    pub text: Option<Vec<u16>>,
    pub uri: Option<Vec<u16>>,
    /// Its MIME types joined by commas, as Room kept them.
    pub mime_types: Vec<u16>,
}

impl Clip {
    /// Whether `other` is the same clip but for its id, as `ClipboardItem`'s equality is.
    pub fn same_as(&self, other: &Clip) -> bool {
        Clip {
            id: 0,
            ..self.clone()
        } == Clip {
            id: 0,
            ..other.clone()
        }
    }

    fn media(&self) -> Option<&[u16]> {
        matches!(self.kind, IMAGE | VIDEO)
            .then_some(self.uri.as_deref())
            .flatten()
    }

    fn policy(&self) -> policy::Clip {
        policy::Clip {
            timestamp: self.created,
            kind: self.kind,
            pinned: self.pinned,
            sensitive: self.sensitive,
            text: self.text.clone(),
        }
    }
}

/// The record of a media file the clipboard keeps, which its content provider serves.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileInfo {
    pub id: i64,
    pub name: Vec<u16>,
    pub size: i64,
    pub orientation: i32,
    pub mime_types: Vec<u16>,
}

/// The size limit, applied after every change.
#[derive(Clone, Copy, Debug)]
pub struct Limit {
    pub now: i64,
    pub enabled: bool,
    pub maximum: i32,
}

/// Which clips expire.
#[derive(Clone, Copy, Debug)]
pub struct Expiry {
    pub old: bool,
    pub minutes: i32,
    pub sensitive: bool,
    pub seconds: i32,
}

/// The clipboard store's value.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Clips {
    /// The id the next clip gets, unless one has a larger id: Room's `AUTOINCREMENT` sequence.
    pub next_id: i64,
    /// The fingerprints of the old databases imported into it ([`crate::legacy`]).
    pub imported: Vec<u64>,
    /// By id, smallest first.
    pub items: Vec<Clip>,
    pub files: Vec<FileInfo>,
}

impl Clips {
    /// The history as shown: newest first, equal times in id order.
    pub fn history(&self) -> Vec<&Clip> {
        self.order().into_iter().map(|at| &self.items[at]).collect()
    }

    fn order(&self) -> Vec<usize> {
        let clips: Vec<_> = self.items.iter().map(Clip::policy).collect();
        policy::order(&clips)
    }

    /// The history as shown, as the policy reads it, with each clip's id.
    fn shown(&self) -> (Vec<policy::Clip>, Vec<i64>) {
        let history = self.history();
        let ids = history.iter().map(|clip| clip.id).collect();
        (history.into_iter().map(Clip::policy).collect(), ids)
    }

    /// Adds `clip` as a new row with the next id, which it returns.
    fn insert(&mut self, mut clip: Clip) -> i64 {
        let id = self.next_id.max(1);
        clip.id = id;
        self.next_id = id + 1;
        self.items.push(clip);
        id
    }

    fn remove(&mut self, keep: impl Fn(&Clip) -> bool) -> Vec<Clip> {
        let (kept, removed) = self.items.drain(..).partition(|clip| keep(clip));
        self.items = kept;
        removed
    }

    /// The media of `removed` that no clip left refers to, each once.
    fn orphans(&self, removed: &[Clip]) -> Vec<Vec<u16>> {
        let mut uris: Vec<Vec<u16>> = Vec::new();
        for uri in removed.iter().filter_map(Clip::media) {
            let used = self.items.iter().any(|clip| clip.media() == Some(uri));
            if !used && !uris.iter().any(|known| known == uri) {
                uris.push(uri.to_vec());
            }
        }
        uris
    }

    /// Copies `clip` into the history, its id ignored; a text the history has already moves to
    /// the top instead, pinned if either was. The new row's id.
    pub fn add(&mut self, mut clip: Clip, limit: Limit) -> i64 {
        let (shown, ids) = self.shown();
        let duplicate = policy::duplicate(&shown, clip.text.as_deref(), true, clip.pinned);
        if let Some((at, pinned)) = duplicate {
            self.remove(|row| row.id != ids[at]);
            clip.pinned = pinned;
        }
        let id = self.insert(clip);
        self.enforce(limit);
        id
    }

    /// Adds each of `clips` the history doesn't already have, in order, as restoring a backup
    /// does; how many it added.
    pub fn restore(&mut self, clips: Vec<Clip>, limit: Limit) -> usize {
        let before = self.items.clone();
        let mut added = 0;
        for clip in clips {
            if !before.iter().any(|row| row.same_as(&clip)) {
                self.insert(clip);
                added += 1;
            }
        }
        self.enforce(limit);
        added
    }

    /// Pins or unpins the clip `id`, if there is one.
    pub fn set_pinned(&mut self, id: i64, pinned: bool, limit: Limit) {
        if let Some(clip) = self.items.iter_mut().find(|clip| clip.id == id) {
            clip.pinned = pinned;
        }
        self.enforce(limit);
    }

    /// Removes the clips `ids`, or only those unpinned; the media no clip refers to any more.
    pub fn delete(&mut self, ids: &[i64], only_unpinned: bool, limit: Limit) -> Vec<Vec<u16>> {
        let removed = self.remove(|clip| !ids.contains(&clip.id) || (only_unpinned && clip.pinned));
        self.enforce(limit);
        self.orphans(&removed)
    }

    /// Removes every clip, or every unpinned one; the media no clip refers to any more.
    pub fn clear(&mut self, unpinned_only: bool, limit: Limit) -> Vec<Vec<u16>> {
        let removed = self.remove(|clip| unpinned_only && clip.pinned);
        self.enforce(limit);
        self.orphans(&removed)
    }

    /// Removes the clips that have expired at `limit.now`.
    pub fn expire(&mut self, expiry: Expiry, limit: Limit) {
        let (shown, ids) = self.shown();
        let (old, minutes) = (expiry.old, expiry.minutes);
        let (sensitive, seconds) = (expiry.sensitive, expiry.seconds);
        let expired = policy::expiry(&shown, limit.now, old, minutes, sensitive, seconds);
        let expired: Vec<i64> = expired.into_iter().map(|at| ids[at]).collect();
        self.remove(|clip| !expired.contains(&clip.id));
        self.enforce(limit);
    }

    /// Removes the oldest unpinned clips over the limit.
    pub fn enforce(&mut self, limit: Limit) {
        let (shown, ids) = self.shown();
        let over = policy::limit(&shown, limit.now, limit.enabled, limit.maximum);
        if !over.is_empty() {
            let over: Vec<i64> = over.into_iter().map(|at| ids[at]).collect();
            self.remove(|clip| !over.contains(&clip.id));
        }
    }

    /// The record of the media file `id`.
    pub fn file(&self, id: i64) -> Option<&FileInfo> {
        self.files.iter().find(|file| file.id == id)
    }

    /// Keeps `file`'s record, replacing any with its id.
    pub fn add_file(&mut self, file: FileInfo) {
        self.remove_file(file.id);
        self.files.push(file);
    }

    pub fn remove_file(&mut self, id: i64) {
        self.files.retain(|file| file.id != id);
    }
}
