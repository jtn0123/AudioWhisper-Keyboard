//! Shared by the tests: the dictionary, keyboard and session a test asks, its golden tape, and
//! [`in_chunks`], which spreads a test's rounds over threads.

use std::path::Path;
use std::sync::{Arc, OnceLock};
use std::thread;

use latin_golden::queries::{self, Random};
use latin_golden::{Answer, Fnv, Suggestions};

use crate::defines::MAX_PROXIMITY_CHARS_SIZE;
use crate::golden::{Tape, tape};
use crate::layout::{KeyboardGeometry, ProximityInfo};
use crate::search::{DicTraverseSession, Output, Query};
use crate::trie::dictionary::Dictionary;

/// The app's QWERTY model (`suggest::qwerty`): three staggered rows of 100 by 150 keys on a
/// 1000 by 450 keyboard, a 32 by 16 grid listing the keys whose edge is within 1.2 key widths of
/// each cell's centre (AOSP's `computeNearestNeighbors`).
pub fn qwerty_info() -> ProximityInfo {
    TestKeyboard::new(false).info()
}

/// A keyboard the tests build: the app's QWERTY rows and, with `space`, a fourth
/// row holding a 500-wide space bar under them, which the app's keyboard model does not have.
pub struct TestKeyboard {
    pub width: i32,
    pub height: i32,
    pub xs: Vec<i32>,
    pub ys: Vec<i32>,
    pub widths: Vec<i32>,
    pub heights: Vec<i32>,
    pub codes: Vec<i32>,
    /// `GRID_WIDTH * GRID_HEIGHT * MAX_PROXIMITY_CHARS_SIZE` code points, -1 for none.
    pub grid: Vec<i32>,
}

impl TestKeyboard {
    pub const KEY_WIDTH: i32 = 100;
    pub const KEY_HEIGHT: i32 = 150;
    pub const GRID_WIDTH: i32 = 32;
    pub const GRID_HEIGHT: i32 = 16;

    pub fn new(space: bool) -> Self {
        let (key_width, key_height) = (Self::KEY_WIDTH, Self::KEY_HEIGHT);
        let rows = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
        let offsets = [0, key_width / 2, key_width * 3 / 2];
        let mut keyboard = Self {
            width: 10 * key_width,
            height: (3 + i32::from(space)) * key_height,
            xs: Vec::new(),
            ys: Vec::new(),
            widths: Vec::new(),
            heights: Vec::new(),
            codes: Vec::new(),
            grid: Vec::new(),
        };
        for (row, letters) in rows.iter().enumerate() {
            for (column, letter) in letters.chars().enumerate() {
                let x = offsets[row] + column as i32 * key_width;
                keyboard.push_key(x, row as i32 * key_height, key_width, letter as i32);
            }
        }
        if space {
            keyboard.push_key(
                5 * key_width / 2,
                3 * key_height,
                5 * key_width,
                i32::from(b' '),
            );
        }
        keyboard.grid = keyboard.proximity_grid();
        keyboard
    }

    fn push_key(&mut self, x: i32, y: i32, width: i32, code: i32) {
        self.xs.push(x);
        self.ys.push(y);
        self.widths.push(width);
        self.heights.push(Self::KEY_HEIGHT);
        self.codes.push(code);
    }

    /// Each cell's keys: those whose edge is within 1.2 key widths of the cell's centre.
    fn proximity_grid(&self) -> Vec<i32> {
        let cell_width = (self.width + Self::GRID_WIDTH - 1) / Self::GRID_WIDTH;
        let cell_height = (self.height + Self::GRID_HEIGHT - 1) / Self::GRID_HEIGHT;
        let threshold = (Self::KEY_WIDTH as f32 * 1.2) as i32;
        let mut grid = Vec::new();
        for cell in 0..Self::GRID_WIDTH * Self::GRID_HEIGHT {
            let center_x = (cell % Self::GRID_WIDTH) * cell_width + cell_width / 2;
            let center_y = (cell / Self::GRID_WIDTH) * cell_height + cell_height / 2;
            let mut slots = Vec::new();
            for key in 0..self.codes.len() {
                let dx = 0
                    .max(self.xs[key] - center_x)
                    .max(center_x - (self.xs[key] + self.widths[key]));
                let dy = 0
                    .max(self.ys[key] - center_y)
                    .max(center_y - (self.ys[key] + self.heights[key]));
                if dx * dx + dy * dy < threshold * threshold {
                    slots.push(self.codes[key]);
                }
            }
            slots.resize(MAX_PROXIMITY_CHARS_SIZE, -1);
            grid.extend(slots);
        }
        grid
    }

    /// The Rust side's `ProximityInfo`.
    pub fn info(&self) -> ProximityInfo {
        ProximityInfo::new(&KeyboardGeometry {
            keyboard_width: self.width,
            keyboard_height: self.height,
            grid_width: Self::GRID_WIDTH,
            grid_height: Self::GRID_HEIGHT,
            most_common_key_width: Self::KEY_WIDTH,
            most_common_key_height: Self::KEY_HEIGHT,
            proximity_chars: &self.grid,
            key_count: self.codes.len() as i32,
            key_x_coordinates: &self.xs,
            key_y_coordinates: &self.ys,
            key_widths: &self.widths,
            key_heights: &self.heights,
            key_char_codes: &self.codes,
        })
        .expect("a valid keyboard")
    }
}

/// Google's AOSP dictionary, read for the Rust side.
pub fn aosp_file() -> Arc<[u8]> {
    std::fs::read(latin_golden::aosp_dictionary_path())
        .expect("the AOSP dictionary")
        .into()
}

/// The Rust dictionary and the keyboard.
pub fn rust_side() -> (Dictionary, ProximityInfo) {
    (
        Dictionary::open(aosp_file()).expect("a valid dictionary"),
        qwerty_info(),
    )
}

/// A session for a dictionary as large as these (the large node cache).
pub fn new_session() -> DicTraverseSession {
    let size = std::fs::metadata(latin_golden::aosp_dictionary_path())
        .expect("the AOSP dictionary")
        .len();
    DicTraverseSession::new(usize::try_from(size).expect("size fits"))
}

/// A generated question as the Rust search takes it.
pub fn convert(query: &queries::Query) -> Query {
    Query {
        code_points: query.code_points.clone(),
        points: query.points.clone(),
        block_offensive_words: query.block_offensive_words,
        previous: query.previous.clone(),
        beginning_of_sentence: query.beginning_of_sentence.clone(),
    }
}

/// The question for the first `length` taps of `query`.
pub fn prefix(query: &queries::Query, length: usize) -> queries::Query {
    queries::Query {
        code_points: query.code_points[..length].to_vec(),
        points: query.points[..length].to_vec(),
        ..query.clone()
    }
}

/// The words of `dictionary`, to draw typed words from.
pub fn words_of(dictionary: &Dictionary) -> Vec<Vec<char>> {
    let text = String::from_utf16(&crate::trie::jni::get_words(dictionary)).expect("UTF-16 words");
    text.lines().map(|it| it.chars().collect()).collect()
}

/// What a test asks: a dictionary, the keyboard it is typed on, and the words to type.
pub struct Fixture {
    pub dictionary: Dictionary,
    pub info: ProximityInfo,
    pub words: Vec<Vec<char>>,
    /// Which dictionary, for the golden files' names.
    pub name: String,
}

impl Fixture {
    /// Google's AOSP dictionary and the one the app ships, one fixture each for every test in
    /// the binary, made by the first that asks.
    pub fn each() -> [&'static Self; 2] {
        static FIXTURES: OnceLock<[Fixture; 2]> = OnceLock::new();
        let [aosp, shipped] = FIXTURES.get_or_init(|| {
            latin_golden::dictionaries().map(|(name, path)| Self::at(name, &path, None))
        });
        [aosp, shipped]
    }

    /// The dictionary at `path` on `keyboard`, or on the app's QWERTY model.
    pub fn at(name: &str, path: &Path, keyboard: Option<&TestKeyboard>) -> Self {
        let bytes = std::fs::read(path).expect("the dictionary");
        let mut fixture = Self::of_bytes(name, bytes);
        if let Some(keyboard) = keyboard {
            fixture.info = keyboard.info();
        }
        fixture
    }

    /// A dictionary from its bytes, on the app's QWERTY model.
    pub fn of_bytes(name: &str, bytes: Vec<u8>) -> Self {
        let dictionary = Dictionary::open(bytes.into()).expect("a valid dictionary");
        Self {
            words: words_of(&dictionary),
            dictionary,
            info: qwerty_info(),
            name: name.to_owned(),
        }
    }

    pub fn session(&self) -> DicTraverseSession {
        new_session()
    }

    /// The tape for `test` on this fixture's dictionary.
    pub fn tape(&self, test: &str) -> Tape {
        tape(&format!("search.{test}.{}", self.name))
    }

    /// One query put to `session`, as the app does; the output.
    pub fn ask(&self, session: &mut DicTraverseSession, query: &queries::Query) -> Output {
        crate::search::suggest(&self.dictionary, &self.info, session, &convert(query))
    }

    /// `query` typed letter by letter on `session`, then whole on a fresh one; the outputs, the
    /// whole query's last.
    pub fn type_then_ask(
        &self,
        session: &mut DicTraverseSession,
        query: &queries::Query,
    ) -> Vec<Output> {
        let mut outputs: Vec<_> = (1..=query.code_points.len())
            .map(|length| self.ask(session, &prefix(query, length)))
            .collect();
        outputs.push(self.ask(&mut self.session(), query));
        outputs
    }

    /// [`Fixture::ask`] with the options array and weight the app never changes.
    pub fn ask_with(
        &self,
        session: &mut DicTraverseSession,
        query: &queries::Query,
        (options, weight): (&[i32], f32),
    ) -> Output {
        let query = convert(query);
        crate::search::suggest_with(
            &self.dictionary,
            &self.info,
            session,
            &query,
            options,
            weight,
        )
    }
}

/// Every field of an output, every array in full and the weight by its bits; a sample shows the
/// suggestions.
impl Answer for Output {
    fn feed(&self, fnv: &mut Fnv) {
        self.count.feed(fnv);
        self.code_points.feed(fnv);
        self.scores.feed(fnv);
        self.space_indices.feed(fnv);
        self.types.feed(fnv);
        self.auto_commit_confidence.feed(fnv);
        self.weight.feed(fnv);
    }

    fn render(&self, out: &mut String) {
        Suggestions {
            count: self.count,
            code_points: &self.code_points,
            scores: &self.scores,
            types: &self.types,
            space_indices: &self.space_indices,
            auto_commit_confidence: self.auto_commit_confidence,
            weight: self.weight,
        }
        .render(out);
    }
}

/// Records every output on `tape`, in order.
pub fn record<'a>(tape: &mut Tape, outputs: impl IntoIterator<Item = &'a Output>) {
    for output in outputs {
        tape.record("suggestions", output);
    }
}

/// The centre of the key for a lower-case ASCII letter on the test keyboard, nudged by up to 30
/// either way; `None` for anything else.
pub fn tap(random: &mut Random, code_point: i32) -> Option<(i32, i32)> {
    let rows = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
    let letter = char::from_u32(code_point as u32)?.to_ascii_lowercase();
    let (row, column) = rows
        .iter()
        .enumerate()
        .find_map(|(row, keys)| keys.find(letter).map(|column| (row, column)))?;
    let nudge = |random: &mut Random| random.below(61) as i32 - 30;
    Some((
        [0, 50, 150][row] + column as i32 * 100 + 50 + nudge(random),
        row as i32 * 150 + 75 + nudge(random),
    ))
}

/// How many threads a test's rounds are spread over. Fixed, so the rounds each thread runs, and
/// the state its session carries from one round to the next, are the same on any computer.
const CHUNKS: usize = 8;

/// `run` for every round of `inputs`, in [`CHUNKS`] runs of consecutive rounds on threads of
/// their own, each run with its own session from `start`; the results in round order. The
/// inputs are made beforehand from the test's one generator, so they are the same however the
/// rounds are split.
pub fn in_chunks<I: Sync, S, R: Send>(
    inputs: &[I],
    start: impl Fn() -> S + Sync,
    run: impl Fn(&mut S, usize, &I) -> R + Sync,
) -> Vec<R> {
    let size = inputs.len().div_ceil(CHUNKS).max(1);
    let (start, run) = (&start, &run);
    thread::scope(|scope| {
        let handles: Vec<_> = inputs
            .chunks(size)
            .enumerate()
            .map(|(chunk, inputs)| {
                scope.spawn(move || {
                    let mut state = start();
                    inputs
                        .iter()
                        .enumerate()
                        .map(|(index, input)| run(&mut state, chunk * size + index, input))
                        .collect::<Vec<R>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|it| {
                it.join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    })
}
