//! The graph's calls as they cross from Android: `wire::key_graph`'s rows read into requests and
//! facts, and answers written back as rows.

use popup::order::{HintConfig, HintMode};
use wire::key_graph::{
    action, action_answer, build, data, facts, file, hints, info, key, question,
};

use crate::data::Data;
use crate::facts::Facts;
use crate::graph::{Files, Graph, Request};

/// Why a call was refused: a row cut short, a number that doesn't parse, or an unknown operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub &'static str);

type Result<T> = std::result::Result<T, Error>;
type Rows = Vec<Vec<u16>>;

/// The graph a `KeyGraphBuild` row describes.
pub fn build(fields: &[Vec<u16>]) -> Result<Graph> {
    if fields.len() != build::SIZE {
        return Err(Error("a build row has one field each"));
    }
    let texts: Vec<String> = fields
        .iter()
        .map(|field| String::from_utf16(field).map_err(|_| Error("a file is not UTF-16")))
        .collect::<Result<_>>()?;
    let loaded: i32 = number(&fields[build::LOADED])?;
    let file = |bit: i32, field: usize| (loaded & bit != 0).then_some(texts[field].as_str());
    let own = |names: i32, bit: i32, field: usize| (loaded & names != 0).then(|| file(bit, field));
    let request = Request {
        mode: number(&fields[build::MODE])?,
        number_hints: wire::flag::decode(&fields[build::NUMBER_HINTS]),
        layouts: Files {
            main: file(file::MAIN, build::MAIN),
            modifier: file(file::MODIFIER, build::MODIFIER),
            own_modifier: own(
                file::NAMES_MODIFIER,
                file::OWN_MODIFIER,
                build::OWN_MODIFIER,
            ),
            extension: file(file::EXTENSION, build::EXTENSION),
        },
        symbols: Files {
            main: file(file::SYMBOLS_MAIN, build::SYMBOLS_MAIN),
            modifier: file(file::SYMBOLS_MODIFIER, build::SYMBOLS_MODIFIER),
            own_modifier: own(
                file::SYMBOLS_NAMES_MODIFIER,
                file::SYMBOLS_OWN_MODIFIER,
                build::SYMBOLS_OWN_MODIFIER,
            ),
            extension: file(file::SYMBOLS_EXTENSION, build::SYMBOLS_EXTENSION),
        },
        mapping: file(file::MAPPING, build::MAPPING),
        fallback: file(file::FALLBACK, build::FALLBACK),
    };
    Ok(Graph::build(&request))
}

/// `operation`'s answer to `fields`, as `wire::key_graph::OPS` describes each.
pub fn call(graph: &mut Graph, operation: &[u16], fields: &[Vec<u16>]) -> Result<Rows> {
    let operation = String::from_utf16(operation).map_err(|_| Error("not an operation"))?;
    use wire::key_graph::*;
    match operation.as_str() {
        COMPUTE => compute(graph, fields),
        ANSWER => graph.answer(fields).map(|()| Vec::new()).map_err(Error),
        KEYS => Ok(keys(graph)),
        POPUPS => popups(graph, fields),
        POPUP_FLAGS => popup_flags(graph, fields),
        CODE_POINTS => {
            let multi: usize = number(field(fields, 0)?)?;
            Ok(graph
                .code_points(multi)
                .iter()
                .map(|&code| text(code))
                .collect())
        }
        INFO => Ok(info(graph)),
        ACTIONS => actions(graph, fields),
        _ => Err(Error("unknown operation")),
    }
}

/// Lays the keys out for `numbers` (width, height, row height, horizontal and vertical margins):
/// the desired key's visible size, then each listed key's touch and visible bounds, enabled
/// state and code.
pub fn layout(graph: &mut Graph, numbers: &[f32]) -> Result<Vec<f32>> {
    let &[width, height, row, horizontal, vertical] = numbers else {
        return Err(Error("a layout takes five numbers"));
    };
    let desired = graph.layout(width, height, row, [horizontal, vertical]);
    let mut out = desired.to_vec();
    for key in graph.keys().iter().take(graph.listed()) {
        let (touch, visible) = (key.bounds.touch, key.bounds.visible);
        out.extend([touch.left, touch.top, touch.right, touch.bottom]);
        out.extend([visible.left, visible.top, visible.right, visible.bottom]);
        out.extend([f32::from(u8::from(key.enabled)), key.data.code as f32]);
    }
    Ok(out)
}

fn compute(graph: &mut Graph, fields: &[Vec<u16>]) -> Result<Rows> {
    let row = fields
        .get(..facts::SIZE)
        .ok_or(Error("a facts row is cut short"))?;
    let slots: usize = number(&row[facts::SLOTS])?;
    let resources: usize = number(&row[facts::RESOURCES])?;
    let rest = &fields[facts::SIZE..];
    if rest.len() != 2 * (slots + resources) {
        return Err(Error("each slot and resource is a pair"));
    }
    let pairs = |rows: &[Vec<u16>]| -> Result<Vec<(i32, Vec<u16>)>> {
        rows.chunks(2)
            .map(|pair| Ok((number(&pair[0])?, pair[1].clone())))
            .collect()
    };
    let flag = |at: usize| wire::flag::decode(&row[at]);
    let facts = Facts {
        state: number(&row[facts::STATE])?,
        input: number(&row[facts::INPUT])?,
        options: number(&row[facts::OPTIONS])?,
        utility: flag(facts::UTILITY),
        utility_action: number(&row[facts::UTILITY_ACTION])?,
        language_switch: flag(facts::LANGUAGE_SWITCH),
        rich_editor: flag(facts::RICH_EDITOR),
        locked: flag(facts::LOCKED),
        pastable: flag(facts::PASTABLE),
        incognito: number(&row[facts::INCOGNITO])?,
        no_learning: flag(facts::NO_LEARNING),
        subtypes: number(&row[facts::SUBTYPES])?,
        dictation: flag(facts::DICTATION),
        language: row[facts::LANGUAGE].clone(),
        default_language: row[facts::DEFAULT_LANGUAGE].clone(),
        slots: pairs(&rest[..2 * slots])?,
        resources: pairs(&rest[2 * slots..])?,
        hints: config(&row[facts::HINT_SYMBOL..=facts::HINT_MERGE])?,
    };
    let questions = graph.compute(&facts);
    Ok(questions
        .into_iter()
        .flat_map(|question| [text(question.ask), text(question.locale), question.value])
        .collect())
}

/// A [`key`] row holds a [`data`] row from `TYPE` to `MULTI`: the same fields, in the same order.
const _: () = {
    assert!(data::TYPE == 0 && key::MULTI - key::TYPE + 1 == data::SIZE);
    assert!(key::TYPE + data::CODE == key::CODE);
    assert!(key::TYPE + data::LABEL == key::LABEL);
    assert!(key::TYPE + data::GROUP == key::GROUP);
    assert!(key::TYPE + data::POPUP == key::POPUP);
    assert!(key::TYPE + data::MULTI == key::MULTI);
};

fn keys(graph: &Graph) -> Rows {
    let bases = graph.base_codes();
    let mut out = Vec::new();
    for (key, base) in graph.keys().iter().zip(bases).take(graph.listed()) {
        let mut row = vec![Vec::new(); key::SIZE];
        row[key::VISIBLE] = wire::flag::utf16(key.visible);
        row[key::ENABLED] = wire::flag::utf16(key.enabled);
        let data = data_row(&key.data);
        row[key::TYPE..=key::MULTI].clone_from_slice(&data);
        row[key::LABELED] = wire::flag::utf16(key.label.is_some());
        row[key::SHOWN] = key.label.clone().unwrap_or_default();
        row[key::HINTED] = wire::flag::utf16(key.hint.is_some());
        row[key::HINT] = key.hint.clone().unwrap_or_default();
        row[key::ICON] = text(key.icon);
        row[key::BASE] = text(base);
        out.extend(row);
    }
    out
}

fn popups(graph: &Graph, fields: &[Vec<u16>]) -> Result<Rows> {
    let at: usize = number(field(fields, 0)?)?;
    let config = config(fields.get(1..).unwrap_or_default())?;
    let keys = graph.popup_keys(at, config);
    let mut out = vec![text(keys.prioritized.len())];
    out.extend(keys.all().flat_map(data_row));
    Ok(out)
}

fn popup_flags(graph: &Graph, fields: &[Vec<u16>]) -> Result<Rows> {
    let config = config(fields)?;
    Ok((0..graph.listed())
        .map(|at| wire::flag::utf16(!graph.popup_keys(at, config).is_empty()))
        .collect())
}

/// What the graph says of each action row's key: enabled, its icon token and its label.
fn actions(graph: &mut Graph, fields: &[Vec<u16>]) -> Result<Rows> {
    if !fields.len().is_multiple_of(action::SIZE) {
        return Err(Error("an action row has one field each"));
    }
    let mut out = Vec::new();
    for row in fields.chunks(action::SIZE) {
        let (kind, code) = (number(&row[action::TYPE])?, number(&row[action::CODE])?);
        let answer = graph.action(kind, code, &row[action::LABEL]);
        let mut answered = vec![Vec::new(); action_answer::SIZE];
        answered[action_answer::ENABLED] = wire::flag::utf16(answer.enabled);
        answered[action_answer::ICON] = text(answer.icon);
        answered[action_answer::LABELED] = wire::flag::utf16(answer.label.is_some());
        answered[action_answer::LABEL] = answer.label.unwrap_or_default();
        out.extend(answered);
    }
    Ok(out)
}

fn info(graph: &Graph) -> Rows {
    let mut out = vec![Vec::new(); info::SIZE];
    out[info::MODE] = text(graph.mode());
    out[info::ROWS] = text(graph.row_sizes().len());
    out[info::LISTED] = text(graph.listed());
    out
}

/// A computed key as a `KeyGraphData` row.
fn data_row(data: &Data) -> Rows {
    let id = |id: Option<usize>| id.map_or_else(|| text(-1), text);
    let mut row = vec![Vec::new(); data::SIZE];
    row[data::TYPE] = text(data.kind);
    row[data::CODE] = text(data.code);
    row[data::LABEL].clone_from(&data.label);
    row[data::GROUP] = text(data.group);
    row[data::POPUP] = id(data.popup);
    row[data::MULTI] = id(data.multi);
    row
}

/// A `KeyGraphHints` row's settings.
fn config(row: &[Vec<u16>]) -> Result<HintConfig> {
    if row.len() != hints::SIZE {
        return Err(Error("a hints row has one field each"));
    }
    Ok(HintConfig {
        symbol: mode(number(&row[hints::SYMBOL])?)?,
        number: mode(number(&row[hints::NUMBER])?)?,
        merge_hint_popups: wire::flag::decode(&row[hints::MERGE]),
    })
}

/// A `KeyHintMode` by its ordinal.
fn mode(ordinal: i32) -> Result<HintMode> {
    Ok(match ordinal {
        0 => HintMode::Disabled,
        1 => HintMode::HintPriority,
        2 => HintMode::AccentPriority,
        3 => HintMode::SmartPriority,
        _ => return Err(Error("not a hint mode")),
    })
}

fn field(fields: &[Vec<u16>], at: usize) -> Result<&[u16]> {
    fields
        .get(at)
        .map(Vec::as_slice)
        .ok_or(Error("a field is missing"))
}

/// A number as Kotlin's `toString` spells it.
fn number<T: std::str::FromStr>(units: &[u16]) -> Result<T> {
    String::from_utf16(units)
        .ok()
        .and_then(|text| text.parse().ok())
        .ok_or(Error("not a number"))
}

fn text(value: impl ToString) -> Vec<u16> {
    value.to_string().encode_utf16().collect()
}

/// Keeps the question fields' order where the wire declares it.
const _: () = assert!(question::ASK == 0 && question::LOCALE == 1 && question::VALUE == 2);
