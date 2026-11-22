//! One computation of a keyboard's keys: `TextKey.compute`, then `computeLabelsAndDrawables`.

use keys::{KeyFlex, models};

use crate::compute::{self, Eval};
use crate::data::{Data, Popups, Target};
use crate::facts::{Facts, Platform, ask, locale};
use crate::graph::Key;
use crate::node::{Arena, Mapping, Node};

/// What one computation of the keys reads.
pub struct Context<'a> {
    pub mode: i32,
    pub arena: &'a Arena,
    pub mapping: Option<&'a Mapping>,
    pub fallback: Option<&'a Mapping>,
    pub facts: &'a Facts,
    pub eval: Eval<'a>,
}

impl Context<'_> {
    /// `TextKey.compute`, then `computeLabelsAndDrawables`.
    ///
    /// A value still waiting on Android's answer stands in as itself, so nothing that depends on
    /// one is asked in the same round: a label not yet lowercased would look up a popup set main
    /// never reaches, and casing that set's keys asks about codes main never cases, which for a
    /// code that isn't a code point throws where main doesn't. The key stops where its answers
    /// run out, and the round its questions are answered in computes it further.
    pub fn key(
        &self,
        key: &mut Key,
        node: &Node,
        hints: Option<&[Option<Data>; 2]>,
        platform: &mut Platform,
    ) {
        let asked = platform.misses();
        let computed = compute::node(node, self.eval, platform);
        key.popups = Popups::default();
        if platform.misses() > asked {
            return;
        }
        match computed.filter(|data| compute::visible(self.facts, data.code)) {
            None => {
                key.data = Data::unspecified();
                key.enabled = false;
                key.visible = false;
                key.flex = KeyFlex::default();
            }
            Some(data) => {
                if let Some(set) = data.popup {
                    self.merge(&mut key.popups, set, Target::Relevant, platform);
                }
                if models::popup_mode(self.mode) {
                    self.extended(key, &data, hints, platform);
                }
                key.enabled = compute::enabled(self.facts, data.code);
                key.visible = true;
                let flex = keys::flex(self.mode, data.code, data.kind == NUMERIC);
                key.flex = KeyFlex {
                    width_factor: flex.width_factor,
                    shrink: flex.shrink,
                    grow: flex.grow,
                };
                key.data = data;
            }
        }
        if platform.misses() == asked {
            self.labels(key, platform);
        }
    }

    /// The popups the popup mappings add for the key's label or group, and its hints'.
    fn extended(
        &self,
        key: &mut Key,
        data: &Data,
        hints: Option<&[Option<Data>; 2]>,
        platform: &mut Platform,
    ) {
        let asked = platform.misses();
        let label = platform.text(ask::LOWER_TEXT, locale::SUBTYPE, &data.label);
        if platform.misses() > asked {
            return;
        }
        let group = models::popup_group(data.group).map(|group| group.encode_utf16().collect());
        let extended: Vec<u16> = group.unwrap_or_else(|| label.clone());
        let sources: Vec<Option<usize>> = (0..5)
            .flat_map(|variation| {
                [false, true].map(|fallback| self.lookup(variation, &extended, fallback))
            })
            .collect();
        let present = sources
            .iter()
            .enumerate()
            .filter(|(_, set)| set.is_some())
            .fold(0u32, |bits, (at, _)| bits | (1 << at));
        let variation = models::read(self.facts.state, 1).unwrap_or(0);
        let source = usize::try_from(models::popup_source(variation, present))
            .ok()
            .and_then(|at| sources[at]);
        if extended != label {
            let specific = self
                .lookup(0, &label, false)
                .or(self.lookup(0, &label, true));
            if let Some(set) = specific {
                self.merge(&mut key.popups, set, Target::Relevant, platform);
            }
        }
        if let Some(set) = source {
            self.merge(&mut key.popups, set, Target::Relevant, platform);
        }
        if data.kind == CHARACTER {
            let [number, symbol] = hints.cloned().unwrap_or_default();
            self.hint(key, data.code, symbol, Target::SymbolPopups, platform);
            self.hint(key, data.code, number, Target::NumberPopups, platform);
        }
    }

    /// `addComputedHints` for one hint.
    fn hint(
        &self,
        key: &mut Key,
        code: i32,
        hint: Option<Data>,
        target: Target,
        platform: &mut Platform,
    ) {
        let Some(hint) = hint.filter(|hint| hint.code != code) else {
            return;
        };
        let evaluated = compute::again(&hint, self.eval);
        let popup = evaluated.as_ref().and_then(|data| data.popup);
        match target {
            Target::SymbolPopups => key.popups.symbol_hint = evaluated,
            _ => key.popups.number_hint = evaluated,
        }
        if let Some(set) = popup {
            self.merge(&mut key.popups, set, target, platform);
        }
        let specific = self
            .lookup(0, &hint.label, false)
            .or(self.lookup(0, &hint.label, true));
        if let Some(set) = specific {
            self.merge(&mut key.popups, set, target, platform);
        }
    }

    /// The popup set a mapping (the subtype's, or the default one if `fallback`) gives `label`
    /// under `variation`.
    fn lookup(&self, variation: usize, label: &[u16], fallback: bool) -> Option<usize> {
        let mapping = if fallback {
            self.fallback
        } else {
            self.mapping
        };
        mapping?.get(variation, label)
    }

    /// Merges popup set `set`'s computed keys into `popups`.
    fn merge(&self, popups: &mut Popups, set: usize, target: Target, platform: &mut Platform) {
        let Some(set) = self.arena.popups.get(set) else {
            return;
        };
        let relevant = set
            .relevant
            .iter()
            .map(|node| compute::node(node, self.eval, platform))
            .collect();
        let main = set
            .main
            .as_ref()
            .and_then(|node| compute::node(node, self.eval, platform));
        popups.merge(relevant, main, target);
    }

    /// `computeLabelsAndDrawables`: the label, the icon, and the hint label and data.
    fn labels(&self, key: &mut Key, platform: &mut Platform) {
        key.label = compute::label(&key.data, self.mode, self.facts, platform);
        key.hint = None;
        key.icon = compute::icon(&key.data, self.mode, self.facts);
        let data = &key.data;
        if data.kind == NUMERIC && self.mode == keys::mode::PHONE {
            key.hint = models::phone_hint(data.code).map(|hint| hint.encode_utf16().collect());
        } else if !data.is_space() || data.kind == NUMERIC {
            let hint = key
                .popups
                .keys(self.facts.hints)
                .hint
                .filter(|hint| !hint.is_space());
            key.hint = hint.as_ref().map(|hint| compute::text(hint, platform));
        }
    }
}

/// The `KeyType` ordinals the graph tells apart.
const CHARACTER: i32 = 0;
const NUMERIC: i32 = 7;
