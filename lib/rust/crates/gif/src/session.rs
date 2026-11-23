//! What the GIF panel shows and the search field the keyboard types into while it is open, as
//! Kotlin's `GifSearch` had it: one panel per process, deciding every step and answering each
//! call (`wire::gif`) with its state and the effects for Kotlin to run in order. Kotlin keeps the
//! network, the recents file's JSON and the threads; a fetch or read that answers after it stopped
//! mattering is told apart by its generation, and ignored.

use wire::gif::{self as ops, effect, media, plan, results as status};

use crate::cache::{self, ResultCache};
use crate::chips::{self, RECENTS, TRENDING};
use crate::giphy::{self, Gif};
use crate::problem;
use crate::rows::{self, Fields, text};
use crate::{recents, reply, search_text};

mod recent;
#[cfg(test)]
mod tests;

/// What the panel shows.
#[derive(Clone, Debug, PartialEq)]
pub enum Results {
    Loading,
    Loaded(Vec<Gif>),
    /// The problem to show.
    Failed(Vec<u16>),
}

/// The fetch in flight: its generation and its search.
#[derive(Debug)]
struct Loading {
    generation: i32,
    search: Vec<u16>,
}

/// The panel's state.
#[derive(Debug)]
pub struct Session {
    available: bool,
    key: Vec<u16>,
    showing: bool,
    typing: bool,
    query: Vec<u16>,
    shown: Vec<u16>,
    results: Results,
    typed_word: Option<Vec<u16>>,
    /// The query when the current delete-key swipe began, so swiping back restores it.
    before_swipe: Vec<u16>,
    generation: i32,
    loading: Option<Loading>,
    keys: ResultCache,
    cached: Vec<(Vec<u16>, Vec<Gif>)>,
    recents: recent::Recents,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    /// A panel showing nothing yet, Trending first, with no GIFs sent and nothing to fetch with.
    pub fn new() -> Self {
        Session {
            available: false,
            key: Vec::new(),
            showing: false,
            typing: false,
            query: Vec::new(),
            shown: crate::utf16(TRENDING),
            results: Results::Loading,
            typed_word: None,
            before_swipe: Vec::new(),
            generation: 0,
            loading: None,
            keys: ResultCache::new(cache::TTL_MS, cache::CAPACITY),
            cached: Vec::new(),
            recents: recent::Recents::default(),
        }
    }

    /// The answer to a `GifOp` at `now`: a `GifPlan` row, then the effects.
    pub fn call(
        &mut self,
        op: &str,
        now: i64,
        fields: &[Vec<u16>],
    ) -> Result<Vec<Vec<u16>>, String> {
        let mut effects = Vec::new();
        let mut fields = Fields::new(fields);
        self.apply(op, now, &mut fields, &mut effects)?;
        let mut answer = vec![
            text(wire::flag::encode(self.showing)),
            text(wire::flag::encode(self.typing)),
            self.query.clone(),
            self.shown.clone(),
        ];
        debug_assert_eq!(answer.len(), plan::SIZE);
        answer.append(&mut effects);
        Ok(answer)
    }

    fn apply(
        &mut self,
        op: &str,
        now: i64,
        fields: &mut Fields,
        effects: &mut Vec<Vec<u16>>,
    ) -> Result<(), String> {
        match op {
            ops::SOURCE => {
                self.available = fields.flag()?;
                self.key = fields.take()?.to_vec();
            }
            ops::ATTACH => self.recents.attach(fields.take()?, effects),
            ops::MERGED => self.publish_chips(effects),
            ops::SHOW => self.show(fields.flag()?, now, effects),
            ops::OFFER => self.offer(fields, effects)?,
            ops::SENT => self.sent(fields, effects)?,
            ops::SNAPSHOT => {
                self.publish_results(effects);
                self.publish_chips(effects);
            }
            ops::FETCHED => self.fetched(fields, now, effects)?,
            ops::LOAD => self.load(fields.take()?, now, effects),
            ops::RESET => {
                self.typing = false;
                self.showing = false;
                effects.push(text(effect::FORGET_PREVIEWS));
            }
            _ => self.edit(op, now, fields, effects)?,
        }
        Ok(())
    }

    /// The search field's operations.
    fn edit(
        &mut self,
        op: &str,
        now: i64,
        fields: &mut Fields,
        effects: &mut Vec<Vec<u16>>,
    ) -> Result<(), String> {
        match op {
            ops::START_TYPING => {
                self.query = if self.trending() {
                    Vec::new()
                } else {
                    self.shown.clone()
                };
                self.typing = true;
            }
            ops::STOP_TYPING => self.typing = false,
            ops::TYPE => self.query = search_text::type_text(&self.query, fields.take()?),
            ops::TYPE_WORD => self.query = search_text::type_word(&self.query, fields.take()?),
            ops::DELETE_LAST => self.query = search_text::delete_last(&self.query, 1).to_vec(),
            ops::DELETE_WORD => self.query = search_text::delete_words(&self.query, 1).to_vec(),
            ops::SWIPE => self.swipe(fields)?,
            ops::SUBMIT => {
                self.typing = false;
                let typed = chars::trim(&self.query).to_vec();
                let search = if typed.is_empty() {
                    crate::utf16(TRENDING)
                } else {
                    typed
                };
                self.load(&search, now, effects);
            }
            _ => return Err(format!("no GIF operation {op}")),
        }
        Ok(())
    }

    fn trending(&self) -> bool {
        self.shown.iter().copied().eq(TRENDING.encode_utf16())
    }

    /// A delete-key swipe reached `count` back from where it began: the query shrinks as it
    /// goes and grows back if the swipe returns.
    fn swipe(&mut self, fields: &mut Fields) -> Result<(), String> {
        let words = fields.flag()?;
        let count = fields.int()?;
        if fields.flag()? {
            self.before_swipe = self.query.clone();
        }
        self.query = if words {
            search_text::delete_words(&self.before_swipe, count).to_vec()
        } else {
            search_text::delete_last(&self.before_swipe, count).to_vec()
        };
        Ok(())
    }

    /// The first time it opens, the panel shows Recents if there are any (no GIPHY call), else
    /// what it showed last.
    fn show(&mut self, show: bool, now: i64, effects: &mut Vec<Vec<u16>>) {
        self.showing = show && self.available;
        if !show || matches!(self.results, Results::Loaded(_)) {
            return;
        }
        let search = if self.trending() && !self.recents.is_empty() {
            crate::utf16(RECENTS)
        } else {
            self.shown.clone()
        };
        self.load(&search, now, effects);
    }

    /// The panel opened after some text: offer its last word as a search, unless the field is
    /// private, where tapping the chip would send it to GIPHY.
    fn offer(&mut self, fields: &mut Fields, effects: &mut Vec<Vec<u16>>) -> Result<(), String> {
        let has_text = fields.flag()?;
        let before = fields.take()?;
        let private = fields.flag()?;
        self.typed_word = if private || !has_text {
            None
        } else {
            search_text::last_word(before).map(<[u16]>::to_vec)
        };
        self.publish_chips(effects);
        Ok(())
    }

    /// A GIF was sent: it goes first in recents, and they are saved, unless incognito.
    fn sent(&mut self, fields: &mut Fields, effects: &mut Vec<Vec<u16>>) -> Result<(), String> {
        let incognito = fields.flag()?;
        let gif = fields.gif()?;
        if incognito {
            return Ok(());
        }
        self.recents.add(gif, recents::CAPACITY);
        self.publish_chips(effects);
        self.recents.save(effects);
        Ok(())
    }

    /// Shows `search`'s GIFs. Each GIPHY call counts against 100 an hour, so the search already
    /// showing (or loading) isn't fetched again, and one made in the last 10 minutes comes from
    /// the cache. A failed one is always tried again.
    fn load(&mut self, search: &[u16], now: i64, effects: &mut Vec<Vec<u16>>) {
        if !self.available {
            return;
        }
        if search.iter().copied().eq(RECENTS.encode_utf16()) {
            return self.show_recents(effects);
        }
        let showing = matches!(self.results, Results::Loaded(_)) || self.loading.is_some();
        if search == self.shown.as_slice() && showing {
            return;
        }
        self.shown = search.to_vec();
        self.cancel(effects);
        if let Some(gifs) = self.cached(search, now) {
            self.results = Results::Loaded(gifs);
            return self.publish_results(effects);
        }
        self.results = Results::Loading;
        self.publish_results(effects);
        self.fetch(search, effects);
    }

    fn fetch(&mut self, search: &[u16], effects: &mut Vec<Vec<u16>>) {
        self.generation = self.generation.wrapping_add(1);
        let url = if search.iter().copied().eq(TRENDING.encode_utf16()) {
            giphy::trending_url(&self.key)
        } else {
            giphy::search_url(&self.key, search)
        };
        effects.extend([text(effect::FETCH), text(self.generation), url]);
        let request = reply::request(media::API).unwrap_or_default();
        effects.extend(request.into_iter().map(text));
        self.loading = Some(Loading {
            generation: self.generation,
            search: search.to_vec(),
        });
    }

    /// Stops the fetch in flight, if any.
    fn cancel(&mut self, effects: &mut Vec<Vec<u16>>) {
        if let Some(loading) = self.loading.take() {
            effects.extend([text(effect::CANCEL), text(loading.generation)]);
        }
    }

    fn show_recents(&mut self, effects: &mut Vec<Vec<u16>>) {
        self.cancel(effects);
        self.shown = crate::utf16(RECENTS);
        self.results = Results::Loaded(self.recents.all().to_vec());
        self.publish_results(effects);
    }

    /// A fetch answered: unless it was cancelled, its GIFs are shown and kept, or its problem.
    fn fetched(
        &mut self,
        fields: &mut Fields,
        now: i64,
        effects: &mut Vec<Vec<u16>>,
    ) -> Result<(), String> {
        let generation = fields.int()?;
        let kind = String::from_utf16_lossy(fields.take()?);
        let status = fields.int()?;
        let body = fields.take()?;
        let answer = reply::results(&kind, status, body)?;
        let current = self
            .loading
            .as_ref()
            .is_some_and(|it| it.generation == generation);
        let Some(loading) = self.loading.take_if(|_| current) else {
            return Ok(());
        };
        self.results = match answer {
            Ok(gifs) => {
                self.keep(&loading.search, gifs.clone(), now);
                Results::Loaded(gifs)
            }
            Err(failure) => Results::Failed(crate::utf16(problem::problem(failure))),
        };
        self.publish_results(effects);
        Ok(())
    }

    /// What `search` returned within the cache's time, or nothing.
    fn cached(&mut self, search: &[u16], now: i64) -> Option<Vec<Gif>> {
        let key = cache::key(search);
        let at = self.cached.iter().position(|(kept, _)| *kept == key);
        if self.keys.get(&key, now) {
            return at.map(|at| self.cached[at].1.clone());
        }
        if let Some(at) = at {
            self.cached.remove(at);
        }
        None
    }

    fn keep(&mut self, search: &[u16], gifs: Vec<Gif>, now: i64) {
        let key = cache::key(search);
        match self.cached.iter_mut().find(|(kept, _)| *kept == key) {
            Some(entry) => entry.1 = gifs,
            None => self.cached.push((key.clone(), gifs)),
        }
        for dropped in self.keys.put(&key, now) {
            self.cached.retain(|(kept, _)| *kept != dropped);
        }
    }

    fn publish_results(&self, effects: &mut Vec<Vec<u16>>) {
        let (code, problem, gifs): (i32, &[u16], &[Gif]) = match &self.results {
            Results::Loading => (status::LOADING, &[], &[]),
            Results::Loaded(gifs) => (status::LOADED, &[], gifs),
            Results::Failed(problem) => (status::FAILED, problem, &[]),
        };
        effects.extend([
            text(effect::RESULTS),
            text(code),
            problem.to_vec(),
            text(gifs.len()),
        ]);
        effects.extend(rows::gifs(gifs));
    }

    /// The quick searches: the word just typed, Recents once a GIF was sent, then the categories.
    fn publish_chips(&self, effects: &mut Vec<Vec<u16>>) {
        let chips = chips::chips(self.typed_word.as_deref(), !self.recents.is_empty());
        effects.extend([text(effect::CHIPS), text(chips.len())]);
        effects.extend(chips);
    }

    /// Recents as saved from another thread: the generation `read` asked for, and the GIFs the
    /// file held, or none if it was missing or damaged.
    pub fn read(&mut self, generation: i32, saved: Option<Vec<Gif>>) {
        self.recents.read(generation, saved, recents::CAPACITY);
    }

    /// What to write for the save of recents `generation`: the file and the GIFs, or nothing if
    /// there is no file, or a newer one replaced it.
    pub fn rows(&self, generation: i32) -> Option<(Vec<u16>, &[Gif])> {
        self.recents.rows(generation)
    }
}
