//! Shared by the tests: both sides' dictionary, keyboard and sessions, the comparison, and
//! [`in_chunks`], which spreads a test's rounds over threads.

use std::path::Path;
use std::sync::{Arc, OnceLock};
use std::thread;

use latin_golden::queries::{self, Random};
use latin_golden::{Answer, Fnv, Suggestions};

use crate::defines::MAX_PROXIMITY_CHARS_SIZE;
#[cfg(feature = "oracle")]
use crate::golden::oracle_query;
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

/// A keyboard both sides can be built from: the app's QWERTY rows and, with `space`, a fourth
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

    /// The C++ side's `ProximityInfo`, from the same arrays.
    #[cfg(feature = "oracle")]
    pub fn cpp_proximity(&self) -> latin_oracle::Proximity {
        latin_oracle::hooks::search::proximity(&latin_oracle::hooks::layout::RawKeyboard {
            keyboard_width: self.width,
            keyboard_height: self.height,
            grid_width: Self::GRID_WIDTH,
            grid_height: Self::GRID_HEIGHT,
            most_common_key_width: Self::KEY_WIDTH,
            most_common_key_height: Self::KEY_HEIGHT,
            proximity_chars: &self.grid,
            key_count: self.codes.len() as i32,
            key_xs: &self.xs,
            key_ys: &self.ys,
            key_widths: &self.widths,
            key_heights: &self.heights,
            key_codes: &self.codes,
        })
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

/// A session for a dictionary as large as these (the large node cache, which both take).
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

/// Every field of the output the same as the C++'s, the weight to the bit.
#[cfg(feature = "oracle")]
pub fn assert_same(ours: &Output, theirs: &latin_oracle::Output, context: &str) {
    assert_eq!(ours.count, theirs.count, "count ({context})");
    assert_eq!(
        ours.code_points, theirs.code_points,
        "code points ({context})"
    );
    assert_eq!(ours.scores, theirs.scores, "scores ({context})");
    assert_eq!(
        ours.space_indices, theirs.space_indices,
        "space indices ({context})"
    );
    assert_eq!(ours.types, theirs.types, "types ({context})");
    assert_eq!(
        ours.auto_commit_confidence, theirs.auto_commit_confidence,
        "auto commit confidence ({context})"
    );
    assert_eq!(
        ours.weight.to_bits(),
        theirs.weight.to_bits(),
        "weight {} against {} ({context})",
        ours.weight,
        theirs.weight
    );
}

/// The words of `dictionary`, to draw typed words from.
pub fn words_of(dictionary: &Dictionary) -> Vec<Vec<char>> {
    let text = String::from_utf16(&crate::trie::jni::get_words(dictionary)).expect("UTF-16 words");
    text.lines().map(|it| it.chars().collect()).collect()
}

/// The C++'s dictionary and keyboard, and the file it opened when it was given bytes.
#[cfg(feature = "oracle")]
struct Cpp {
    dictionary: latin_oracle::Dictionary,
    proximity: latin_oracle::Proximity,
    _file: Option<TemporaryFile>,
}

/// A file the C++ side opens, removed when dropped.
#[cfg(feature = "oracle")]
struct TemporaryFile(std::path::PathBuf);

#[cfg(feature = "oracle")]
impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Both sides of a test: the Rust dictionary and keyboard, with the C++ built the C++'s, and the
/// words to type.
pub struct Pair {
    pub dictionary: Dictionary,
    pub info: ProximityInfo,
    pub words: Vec<Vec<char>>,
    /// Which dictionary, for the golden files' names.
    pub name: String,
    #[cfg(feature = "oracle")]
    cpp: Cpp,
}

/// One session each side, kept across calls as the keyboard keeps one.
pub struct Sessions {
    pub ours: DicTraverseSession,
    #[cfg(feature = "oracle")]
    theirs: latin_oracle::Session,
}

impl Pair {
    /// Google's AOSP dictionary and the one the app ships, one pair each for every test in the
    /// binary, made by the first that asks.
    pub fn each() -> [&'static Self; 2] {
        static PAIRS: OnceLock<[Pair; 2]> = OnceLock::new();
        let [aosp, shipped] = PAIRS.get_or_init(|| {
            latin_golden::dictionaries().map(|(name, path)| Self::at(name, &path, None))
        });
        [aosp, shipped]
    }

    /// The dictionary at `path` on `keyboard`, or on the app's QWERTY model.
    pub fn at(name: &str, path: &Path, keyboard: Option<&TestKeyboard>) -> Self {
        let bytes = std::fs::read(path).expect("the dictionary");
        let dictionary = Dictionary::open(bytes.into()).expect("a valid dictionary");
        Self {
            words: words_of(&dictionary),
            dictionary,
            info: keyboard.map_or_else(qwerty_info, TestKeyboard::info),
            name: name.to_owned(),
            #[cfg(feature = "oracle")]
            cpp: Cpp {
                dictionary: latin_oracle::Dictionary::at(path),
                proximity: keyboard
                    .map_or_else(latin_oracle::Proximity::qwerty, |it| it.cpp_proximity()),
                _file: None,
            },
        }
    }

    /// A dictionary from its bytes, on the app's QWERTY model; the C++ opens a copy in a file.
    pub fn of_bytes(name: &str, bytes: Vec<u8>) -> Self {
        #[cfg(feature = "oracle")]
        let file = TemporaryFile(
            std::env::temp_dir().join(format!("latin-bytes-{name}-{}.dict", std::process::id())),
        );
        #[cfg(feature = "oracle")]
        std::fs::write(&file.0, &bytes).expect("a temporary file");
        #[cfg(feature = "oracle")]
        let size = i64::try_from(bytes.len()).expect("size fits");
        let dictionary = Dictionary::open(bytes.into()).expect("a valid dictionary");
        Self {
            words: words_of(&dictionary),
            dictionary,
            info: qwerty_info(),
            name: name.to_owned(),
            #[cfg(feature = "oracle")]
            cpp: Cpp {
                dictionary: latin_oracle::Dictionary::open(&file.0, 0, size).expect("opens"),
                proximity: latin_oracle::Proximity::qwerty(),
                _file: Some(file),
            },
        }
    }

    pub fn sessions(&self) -> Sessions {
        Sessions {
            ours: new_session(),
            #[cfg(feature = "oracle")]
            theirs: latin_oracle::Session::new("en_US", &self.cpp.dictionary),
        }
    }

    /// The tape for `test` on this pair's dictionary.
    pub fn tape(&self, test: &str) -> Tape {
        tape(&format!("search.{test}.{}", self.name))
    }

    /// One query put to both sessions, as the app does, and compared; the Rust output.
    pub fn compare(&self, sessions: &mut Sessions, query: &queries::Query) -> Output {
        let ours = crate::search::suggest(
            &self.dictionary,
            &self.info,
            &mut sessions.ours,
            &convert(query),
        );
        #[cfg(feature = "oracle")]
        {
            let cpp = &self.cpp;
            let theirs = latin_oracle::suggest(
                &cpp.dictionary,
                &cpp.proximity,
                &sessions.theirs,
                &oracle_query(query),
            );
            assert_same(&ours, &theirs, &format!("{}: {query:?}", self.name));
        }
        ours
    }

    /// `query` typed letter by letter on `sessions`, then whole on fresh ones, each compared; the
    /// outputs, the whole query's last.
    pub fn type_then_ask(&self, sessions: &mut Sessions, query: &queries::Query) -> Vec<Output> {
        let mut outputs: Vec<_> = (1..=query.code_points.len())
            .map(|length| self.compare(sessions, &prefix(query, length)))
            .collect();
        outputs.push(self.compare(&mut self.sessions(), query));
        outputs
    }

    /// [`Pair::compare`] with the options array and weight the app never changes.
    pub fn compare_with(
        &self,
        sessions: &mut Sessions,
        query: &queries::Query,
        (options, weight): (&[i32], f32),
    ) -> Output {
        let session = &mut sessions.ours;
        let ours = crate::search::suggest_with(
            &self.dictionary,
            &self.info,
            session,
            &convert(query),
            options,
            weight,
        );
        #[cfg(feature = "oracle")]
        {
            let cpp = &self.cpp;
            let theirs = latin_oracle::hooks::search::suggest_with(
                &cpp.dictionary,
                &cpp.proximity,
                &sessions.theirs,
                &oracle_query(query),
                options,
                weight,
            );
            let context = format!("{}: {query:?} {options:?} {weight}", self.name);
            assert_same(&ours, &theirs, &context);
        }
        ours
    }
}

/// Every field of an output, every array in full and the weight by its bits, as [`assert_same`]
/// compares them; a sample shows the suggestions.
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
/// the state its sessions carry from one round to the next, are the same on any computer.
const CHUNKS: usize = 8;

/// `run` for every round of `inputs`, in [`CHUNKS`] runs of consecutive rounds on threads of
/// their own, each run with its own sessions from `start`; the results in round order. The
/// inputs are made beforehand from the test's one generator, so they are the same however the
/// rounds are split, and only the C++ call itself holds the oracle's lock.
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
