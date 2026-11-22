//! One keyboard's key graph: its layouts read and merged (`LayoutManager.mergeLayouts`), and its
//! keys computed for the keyboard's state (`TextKey.compute` and `computeLabelsAndDrawables`).

use keys::{Bounds, KeyFlex, assembly};
use popup::order::HintConfig;

use crate::assemble::{Reader, assemble, sizes};
use crate::compute::{self, Eval};
use crate::context::Context;
use crate::data::{Data, PopupKeys, Popups};
use crate::facts::{Facts, Platform, Question, locale};
use crate::node::{Arena, Leaf, Mapping, Node};

/// The files of one keyboard's main, modifier and extension layouts, as Kotlin read them;
/// `None` for one that didn't load.
#[derive(Clone, Copy, Debug, Default)]
pub struct Files<'a> {
    pub main: Option<&'a str>,
    pub modifier: Option<&'a str>,
    /// The modifier the main layout's index entry names, when it names one: used instead of
    /// `modifier` if the main layout reads.
    pub own_modifier: Option<Option<&'a str>>,
    pub extension: Option<&'a str>,
}

/// What a keyboard is built from.
#[derive(Clone, Copy, Debug, Default)]
pub struct Request<'a> {
    /// The `KeyboardMode` value.
    pub mode: i32,
    /// The hinted number row setting when the keyboard is built.
    pub number_hints: bool,
    pub layouts: Files<'a>,
    /// The symbols keyboard's files, which a characters keyboard takes its hints from.
    pub symbols: Files<'a>,
    /// The subtype's popup mapping, and the default one behind it.
    pub mapping: Option<&'a str>,
    pub fallback: Option<&'a str>,
}

/// One key's computed state, `TextKey`'s fields; a new key's are those of a `TextKey` never
/// computed.
#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    pub data: Data,
    pub visible: bool,
    pub enabled: bool,
    pub flex: KeyFlex,
    pub popups: Popups,
    pub label: Option<Vec<u16>>,
    pub hint: Option<Vec<u16>>,
    pub hint_data: Data,
    /// The icon token Android draws (`compute::DRAWN`), 0 for none.
    pub icon: i32,
    pub bounds: Bounds,
}

impl Default for Key {
    fn default() -> Self {
        Key {
            data: Data::unspecified(),
            visible: true,
            enabled: true,
            flex: KeyFlex::default(),
            popups: Popups::default(),
            label: None,
            hint: None,
            hint_data: Data::unspecified(),
            icon: 0,
            bounds: Bounds::default(),
        }
    }
}

/// What the smartbar's evaluator says of an action's key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Action {
    pub enabled: bool,
    /// The icon token Android draws, 0 for none.
    pub icon: i32,
    pub label: Option<Vec<u16>>,
}

/// A built keyboard.
#[derive(Clone, Debug, Default)]
pub struct Graph {
    mode: i32,
    arena: Arena,
    rows: Vec<Vec<Node>>,
    /// The symbols keyboard a characters keyboard's hints come from, and whether it hints
    /// numbers; `None` for any other keyboard, or one without rows.
    symbols: Option<(Vec<Vec<Node>>, bool)>,
    /// Each key's number and symbol hint, as the default evaluator computes them, once they are.
    hints: Option<Vec<[Option<Data>; 2]>>,
    mapping: Option<Mapping>,
    fallback: Option<Mapping>,
    platform: Platform,
    keys: Vec<Key>,
    facts: Option<Facts>,
}

impl Graph {
    /// The keyboard `request` describes.
    pub fn build(request: &Request) -> Graph {
        let mut reader = Reader::default();
        let rows = assemble(&request.layouts, &mut reader);
        let characters = request.mode == keys::mode::CHARACTERS && !rows.is_empty();
        let symbols = characters.then(|| {
            (
                assemble(&request.symbols, &mut reader),
                request.number_hints,
            )
        });
        let mapping = request.mapping.and_then(|text| reader.mapping(text));
        let fallback = request.fallback.and_then(|text| reader.mapping(text));
        let count = rows.iter().map(Vec::len).sum();
        Graph {
            mode: request.mode,
            arena: reader.arena,
            rows,
            symbols,
            hints: None,
            mapping,
            fallback,
            platform: Platform::default(),
            keys: vec![Key::default(); count],
            facts: None,
        }
    }

    /// FlorisBoard's `PlaceholderLoadingKeyboard`, shown before any keyboard is built: never
    /// computed.
    pub fn placeholder() -> Graph {
        let key = |code: i32, kind: i32, label: &str| {
            Node::Text(Leaf {
                kind,
                code,
                label: label.encode_utf16().collect(),
                ..Leaf::unspecified()
            })
        };
        let blank = || key(0, 0, "");
        let mut rows = vec![vec![], vec![], vec![], vec![]];
        rows[0].extend((0..10).map(|_| blank()));
        rows[1].extend((0..9).map(|_| blank()));
        rows[2].push(key(keys::codes::SHIFT, 4, "shift"));
        rows[2].extend((0..7).map(|_| blank()));
        rows[2].push(key(keys::codes::DELETE, 1, "delete"));
        rows[3].push(key(keys::codes::VIEW_SYMBOLS, 6, "view_symbols"));
        rows[3].extend([
            blank(),
            blank(),
            key(keys::codes::SPACE, 0, "space"),
            blank(),
        ]);
        rows[3].push(key(keys::codes::ENTER, 1, "enter"));
        let count = rows.iter().map(Vec::len).sum();
        Graph {
            mode: keys::mode::CHARACTERS,
            rows,
            keys: vec![Key::default(); count],
            ..Graph::default()
        }
    }

    pub fn mode(&self) -> i32 {
        self.mode
    }

    /// Each row's number of keys.
    pub fn row_sizes(&self) -> Vec<usize> {
        self.rows.iter().map(Vec::len).collect()
    }

    /// How many keys the keyboard lists: those before its first empty row, as
    /// `TextKeyboardIterator` stops there. Only these are computed and drawn.
    pub fn listed(&self) -> usize {
        self.rows
            .iter()
            .take_while(|row| !row.is_empty())
            .map(Vec::len)
            .sum()
    }

    /// Every key, in row order.
    pub fn keys(&self) -> &[Key] {
        &self.keys
    }

    /// The code of the key a key's layout entry is, before any state computes it (a selector's
    /// is 0): what glide typing indexes keys by.
    pub fn base_codes(&self) -> Vec<i32> {
        let code = |node: &Node| match node {
            Node::Text(leaf) | Node::Auto(leaf, _) | Node::Multi(leaf, _) => leaf.code,
            Node::Select(..) => 0,
        };
        self.rows.iter().flatten().map(code).collect()
    }

    /// A multi-text key's code points.
    pub fn code_points(&self, multi: usize) -> &[i32] {
        self.arena.multis.get(multi).map_or(&[], Vec::as_slice)
    }

    /// The facts of the last complete computation.
    pub fn facts(&self) -> Option<&Facts> {
        self.facts.as_ref()
    }

    /// The platform's answers, for questions asked outside a computation (the smartbar's).
    pub fn platform(&mut self) -> &mut Platform {
        &mut self.platform
    }

    /// Computes every listed key for `facts`. Answers questions the platform still owes instead,
    /// leaving the keys as they were: answer them with [`Graph::answer`] and compute again.
    pub fn compute(&mut self, facts: &Facts) -> Vec<Question> {
        self.platform.forget_pending();
        let hints = self.hints();
        let context = Context {
            mode: self.mode,
            arena: &self.arena,
            mapping: self.mapping.as_ref(),
            fallback: self.fallback.as_ref(),
            facts,
            eval: Eval {
                state: facts.state,
                slots: Some(&facts.slots),
                locale: locale::SUBTYPE,
            },
        };
        let mut keys = self.keys.clone();
        let nodes = self.rows.iter().flatten();
        for (at, (key, node)) in keys.iter_mut().zip(nodes).take(self.listed()).enumerate() {
            let hint = hints.as_ref().and_then(|hints| hints.get(at));
            context.key(key, node, hint, &mut self.platform);
        }
        let pending = self.platform.pending().to_vec();
        if pending.is_empty() && hints.is_some() {
            self.keys = keys;
            self.hints = hints;
            self.facts = Some(facts.clone());
        }
        pending
    }

    /// Takes the answers to the questions the last computation asked.
    pub fn answer(&mut self, answers: &[Vec<u16>]) -> Result<(), &'static str> {
        self.platform.answer(answers)
    }

    /// The hints, computed by the default evaluator the first time; `None` while that still
    /// asks questions.
    fn hints(&mut self) -> Option<Vec<[Option<Data>; 2]>> {
        if let Some(hints) = &self.hints {
            return Some(hints.clone());
        }
        let Some((symbols, number_hints)) = &self.symbols else {
            return Some(vec![[None, None]; self.keys.len()]);
        };
        let eval = Eval {
            state: 0,
            slots: None,
            locale: locale::DEFAULT,
        };
        let computed: Vec<Option<Data>> = symbols
            .iter()
            .flatten()
            .map(|node| compute::node(node, eval, &mut self.platform))
            .collect();
        let types: Vec<i32> = computed
            .iter()
            .map(|data| data.as_ref().map_or(-1, |data| data.kind))
            .collect();
        let indices = assembly::hints(
            self.mode,
            *number_hints,
            &sizes(&self.rows),
            &sizes(symbols),
            &types,
        )
        .unwrap_or_default();
        let pick = |at: i32| {
            usize::try_from(at)
                .ok()
                .and_then(|at| computed.get(at))
                .cloned()
                .flatten()
        };
        let hints = indices
            .chunks(2)
            .map(|pair| [pick(pair[0]), pick(pair[1])])
            .collect();
        self.platform.pending().is_empty().then_some(hints)
    }

    /// What the smartbar asks of an action's key (a `KeyType` ordinal, a code and a label): as
    /// the smartbar's evaluator answers, for a keyboard of mode `SMARTBAR_QUICK_ACTIONS` and the
    /// default subtype, with the facts of the last computation.
    pub fn action(&mut self, kind: i32, code: i32, label: &[u16]) -> Action {
        let facts = self.facts.clone().unwrap_or_default();
        let facts = Facts {
            language: facts.default_language.clone(),
            ..facts
        };
        let data = Data {
            kind,
            code,
            label: label.to_vec(),
            group: 0,
            popup: None,
            multi: None,
        };
        let mode = SMARTBAR_QUICK_ACTIONS;
        Action {
            enabled: compute::enabled(&facts, code),
            icon: compute::icon(&data, mode, &facts),
            label: compute::label(&data, mode, &facts, &mut self.platform),
        }
    }

    /// A key's popup keys for the hint settings `config`.
    pub fn popup_keys(&self, at: usize, config: HintConfig) -> PopupKeys {
        self.keys
            .get(at)
            .map(|key| key.popups.keys(config))
            .unwrap_or_default()
    }

    /// Lays the keys out on a keyboard `width` by `height` (`sizeDesiredKey`, then `layout` with
    /// the touch bounds extended down): the desired key's visible width and height. A layout the
    /// Kotlin skipped leaves each key's bounds as they were.
    pub fn layout(
        &mut self,
        width: f32,
        height: f32,
        row_height: f32,
        margins: [f32; 2],
    ) -> [f32; 2] {
        let rows = self.rows.len() as i32;
        let (touch, visible) = keys::desired_key(
            self.mode, width, height, rows, row_height, margins[0], margins[1],
        );
        let flex: Vec<KeyFlex> = self.keys.iter().map(|key| key.flex).collect();
        let mut rest = flex.as_slice();
        let mut rows = Vec::new();
        for size in self.row_sizes() {
            let (row, after) = rest.split_at(size);
            rows.push(row);
            rest = after;
        }
        if let Some(bounds) = keys::layout(&rows, touch, visible, width, height, true) {
            for (key, bounds) in self.keys.iter_mut().zip(bounds) {
                key.bounds = bounds;
            }
        }
        [visible.width(), visible.height()]
    }
}

/// `KeyboardMode.SMARTBAR_QUICK_ACTIONS`'s value: the smartbar evaluator's keyboard.
const SMARTBAR_QUICK_ACTIONS: i32 = 10;
