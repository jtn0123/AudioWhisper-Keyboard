//! The GIFs sent before, newest first, as Kotlin's `GifRecents` had them: read from their file
//! on another thread, keeping any sent meanwhile, and saved whole after each send. Attaching
//! another file starts a new generation, so a read of the old one changes nothing.

use wire::gif::effect;

use crate::giphy::Gif;
use crate::recents::{self, From};
use crate::rows::text;

#[derive(Debug, Default)]
pub struct Recents {
    gifs: Vec<Gif>,
    path: Option<Vec<u16>>,
    generation: i32,
}

impl Recents {
    pub fn all(&self) -> &[Gif] {
        &self.gifs
    }

    pub fn is_empty(&self) -> bool {
        self.gifs.is_empty()
    }

    /// The keyboard started with recents at `path`: read them, unless that file is the one
    /// already read. Another file starts empty.
    pub fn attach(&mut self, path: &[u16], effects: &mut Vec<Vec<u16>>) {
        if self.path.as_deref() == Some(path) {
            return;
        }
        self.path = Some(path.to_vec());
        self.gifs.clear();
        self.generation = self.generation.wrapping_add(1);
        effects.extend([text(effect::READ), text(self.generation), path.to_vec()]);
    }

    /// `gif` was sent: it goes first, once.
    pub fn add(&mut self, gif: Gif, capacity: usize) {
        let ids: Vec<&[u16]> = self.gifs.iter().map(|it| it.id.as_slice()).collect();
        let order = recents::add(&ids, &gif.id, capacity);
        let before = std::mem::take(&mut self.gifs);
        self.gifs = order
            .into_iter()
            .map(|from| match from {
                From::Before(at) => before[at].clone(),
                From::Sent(_) => gif.clone(),
            })
            .collect();
    }

    /// Generation `generation`'s file held `saved`, or was missing or damaged: keep what was
    /// sent while it was read.
    pub fn read(&mut self, generation: i32, saved: Option<Vec<Gif>>, capacity: usize) {
        let Some(saved) = saved.filter(|_| generation == self.generation) else {
            return;
        };
        let ids = |gifs: &[Gif]| gifs.iter().map(|it| it.id.clone()).collect::<Vec<_>>();
        let order = recents::load(&ids(&saved), &ids(&self.gifs), capacity);
        let sent = std::mem::take(&mut self.gifs);
        self.gifs = order
            .into_iter()
            .map(|from| match from {
                From::Before(at) => saved[at].clone(),
                From::Sent(at) => sent[at].clone(),
            })
            .collect();
    }

    /// Save them: Kotlin asks for [`Recents::rows`] on another thread.
    pub fn save(&self, effects: &mut Vec<Vec<u16>>) {
        effects.extend([text(effect::SAVE), text(self.generation)]);
    }

    /// The file and the GIFs to write for a save of `generation`, or nothing without a file, or
    /// once another file replaced it.
    pub fn rows(&self, generation: i32) -> Option<(Vec<u16>, &[Gif])> {
        let path = self
            .path
            .clone()
            .filter(|_| generation == self.generation)?;
        Some((path, &self.gifs))
    }
}
