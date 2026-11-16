//! Shared by the tests: both sides' dictionary, keyboard and sessions, the comparison, and
//! [`in_chunks`], which spreads a test's rounds over threads.

use std::sync::Arc;
#[cfg(feature = "oracle")]
use std::sync::OnceLock;
#[cfg(feature = "oracle")]
use std::thread;

#[cfg(feature = "oracle")]
use latin_oracle::queries::Random;

use crate::defines::MAX_PROXIMITY_CHARS_SIZE;
use crate::layout::{KeyboardGeometry, ProximityInfo};
use crate::search::DicTraverseSession;
#[cfg(feature = "oracle")]
use crate::search::{Output, Query};
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

/// Google's AOSP dictionary, the fixture `latin_oracle::aosp_dictionary_path` names, available
/// to Rust-only tests without linking the C++ oracle.
fn aosp_dictionary_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/aosp_main_en-US.dict")
}

/// The AOSP dictionary, read for the Rust side.
pub fn aosp_file() -> Arc<[u8]> {
    std::fs::read(aosp_dictionary_path())
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
    let size = std::fs::metadata(aosp_dictionary_path())
        .expect("the AOSP dictionary")
        .len();
    DicTraverseSession::new(usize::try_from(size).expect("size fits"))
}

/// The oracle's query as the Rust search takes it.
#[cfg(feature = "oracle")]
pub fn convert(query: &latin_oracle::Query) -> Query {
    Query {
        code_points: query.code_points.clone(),
        points: query.points.clone(),
        block_offensive_words: query.block_offensive_words,
        previous: query.previous.clone(),
        beginning_of_sentence: query.beginning_of_sentence.clone(),
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
#[cfg(feature = "oracle")]
pub fn words_of(dictionary: &latin_oracle::Dictionary) -> Vec<Vec<char>> {
    let text = String::from_utf16(&dictionary.words()).expect("UTF-16 words");
    text.lines().map(|it| it.chars().collect()).collect()
}

/// Both sides of a differential test: the C++ dictionary and keyboard, the Rust ones, and the
/// words to type.
#[cfg(feature = "oracle")]
pub struct Pair {
    pub cpp_dictionary: latin_oracle::Dictionary,
    pub cpp_proximity: latin_oracle::Proximity,
    pub dictionary: Dictionary,
    pub info: ProximityInfo,
    pub words: Vec<Vec<char>>,
    /// Which dictionary, for failure messages.
    pub name: String,
}

#[cfg(feature = "oracle")]
impl Pair {
    /// Google's AOSP dictionary and the one the app ships (`latin_oracle::dictionary_paths`), one
    /// pair each for every test in the binary, made by the first that asks.
    pub fn each() -> [&'static Self; 2] {
        static PAIRS: OnceLock<[Pair; 2]> = OnceLock::new();
        let [aosp, shipped] =
            PAIRS.get_or_init(|| latin_oracle::dictionary_paths().map(|path| Self::new(&path)));
        [aosp, shipped]
    }

    fn new(path: &std::path::Path) -> Self {
        let file: Arc<[u8]> = std::fs::read(path).expect("the dictionary").into();
        let cpp_dictionary = latin_oracle::Dictionary::at(path);
        let words = words_of(&cpp_dictionary);
        Self {
            cpp_dictionary,
            cpp_proximity: latin_oracle::Proximity::qwerty(),
            dictionary: Dictionary::open(file).expect("a valid dictionary"),
            info: qwerty_info(),
            words,
            name: path
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
        }
    }

    pub fn cpp_session(&self) -> latin_oracle::Session {
        latin_oracle::Session::new("en_US", &self.cpp_dictionary)
    }

    /// One query put to both sessions, as the app does, and compared; the Rust output.
    pub fn compare(
        &self,
        sessions: (&mut DicTraverseSession, &latin_oracle::Session),
        query: &latin_oracle::Query,
        context: &str,
    ) -> Output {
        let theirs =
            latin_oracle::suggest(&self.cpp_dictionary, &self.cpp_proximity, sessions.1, query);
        let ours =
            crate::search::jni::suggest(&self.dictionary, &self.info, sessions.0, &convert(query));
        assert_same(&ours, &theirs, &format!("{}: {context}", self.name));
        ours
    }
}

/// The centre of the key for a lower-case ASCII letter on the test keyboard, nudged by up to 30
/// either way; `None` for anything else.
#[cfg(feature = "oracle")]
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
#[cfg(feature = "oracle")]
const CHUNKS: usize = 8;

/// `run` for every round of `inputs`, in [`CHUNKS`] runs of consecutive rounds on threads of
/// their own, each run with its own sessions from `start`; the results in round order. The
/// inputs are made beforehand from the test's one generator, so they are the same however the
/// rounds are split, and only the C++ call itself holds the oracle's lock.
#[cfg(feature = "oracle")]
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
