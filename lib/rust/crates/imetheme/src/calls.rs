//! The theme's calls as they cross from Android: `wire::ime_theme`'s rows read into facts and
//! the index, and steps written back as rows.

use wire::ime_theme::{accent, chip, component, extension, inline, kind, loaded, step, update};

use crate::Text;
use crate::choose::{Facts, Mode, Name};
use crate::inline::Palette;
use crate::session::{Component, Extension, Index, Session, Step};

/// Why a call was refused: a row cut short, a number that doesn't parse, or an unknown operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub &'static str);

type Result<T> = std::result::Result<T, Error>;
type Rows = Vec<Text>;

/// `operation`'s answer to `fields`, as `wire::ime_theme::OPS` describes each.
pub fn call(session: &mut Session, operation: &[u16], fields: &[Text]) -> Result<Rows> {
    let step = match utf8(operation)?.as_str() {
        wire::ime_theme::UPDATE => {
            let row = fields
                .get(..update::SIZE)
                .ok_or(Error("an update row is cut short"))?;
            let (facts, index) = (facts(row)?, index(row, &fields[update::SIZE..])?);
            let clear = wire::flag::decode(&row[update::CLEAR]);
            let preview = wire::flag::decode(&row[update::PREVIEW_INFO]);
            return Ok(write(
                &session.update(clear, preview, &facts, &index),
                clear,
            ));
        }
        wire::ime_theme::LOADED => {
            let ok = wire::flag::decode(field(fields, loaded::OK)?);
            session.loaded(ok).ok_or(Error("no load is in progress"))?
        }
        _ => return Err(Error("not an operation")),
    };
    Ok(write(&step, false))
}

/// `rule`'s answer to `fields`, as `wire::ime_theme::RULES` describes each.
pub fn rule(rule: &[u16], fields: &[Text]) -> Result<Rows> {
    match utf8(rule)?.as_str() {
        wire::ime_theme::BASE => Ok(vec![snygg::text(crate::base::BASE)]),
        wire::ime_theme::INLINE => {
            let row = fields
                .get(..inline::SIZE)
                .ok_or(Error("an inline row is cut short"))?;
            let json = String::from_utf16(&row[inline::STYLESHEET])
                .map_err(|_| Error("a stylesheet is not UTF-16"))?;
            let palette = palette(&fields[inline::SIZE..])?;
            let colors =
                crate::inline::chip(&json, &row[inline::MODE], &row[inline::SHIFT], &palette);
            let mut out = vec![Text::new(); chip::SIZE];
            out[chip::BACKGROUND] = number_text(colors.0 as i32);
            out[chip::FOREGROUND] = number_text(colors.1 as i32);
            Ok(out)
        }
        wire::ime_theme::PRIMARY => {
            let row = fields
                .get(..accent::SIZE)
                .ok_or(Error("an accent row is cut short"))?;
            let chosen = if wire::flag::decode(&row[accent::SPECIFIED]) {
                accent::PREFERRED
            } else {
                accent::SYSTEM
            };
            Ok(vec![number_text(number::<i32>(&row[chosen])?)])
        }
        _ => Err(Error("not a rule")),
    }
}

fn facts(row: &[Text]) -> Result<Facts> {
    let name = |extension: usize, component: usize| Name {
        extension: row[extension].clone(),
        component: row[component].clone(),
    };
    let preview = wire::flag::decode(&row[update::PREVIEW]);
    Ok(Facts {
        mode: Mode::parse(&utf8(&row[update::MODE])?).ok_or(Error("not a theme mode"))?,
        day: name(update::DAY_EXTENSION, update::DAY_COMPONENT),
        night: name(update::NIGHT_EXTENSION, update::NIGHT_COMPONENT),
        sunrise: number(&row[update::SUNRISE])?,
        sunset: number(&row[update::SUNSET])?,
        now: number(&row[update::NOW])?,
        system_night: wire::flag::decode(&row[update::SYSTEM_NIGHT]),
        preview: preview.then(|| name(update::PREVIEW_EXTENSION, update::PREVIEW_COMPONENT)),
    })
}

fn index(row: &[Text], rest: &[Text]) -> Result<Index> {
    let keyboards: usize = number(&row[update::KEYBOARDS])?;
    let mut rows = rest
        .get(keyboards..)
        .ok_or(Error("the keyboard ids are cut short"))?;
    let mut themes = vec![];
    for _ in 0..number::<usize>(&row[update::THEMES])? {
        let head = take(&mut rows, extension::SIZE)?;
        let mut components = vec![];
        for _ in 0..number::<usize>(&head[extension::COMPONENTS])? {
            let c = take(&mut rows, component::SIZE)?;
            components.push(Component {
                id: c[component::ID].clone(),
                stylesheet: c[component::STYLESHEET].clone(),
            });
        }
        themes.push(Extension {
            id: head[extension::ID].clone(),
            source: wire::flag::decode(&head[extension::SOURCE]),
            components,
        });
    }
    if !rows.is_empty() {
        return Err(Error("an update has rows left over"));
    }
    Ok(Index {
        keyboards: rest[..keyboards].to_vec(),
        themes,
    })
}

fn palette(rows: &[Text]) -> Result<Palette> {
    if !rows.len().is_multiple_of(3) {
        return Err(Error(
            "a palette has a light and a dark colour for each name",
        ));
    }
    let n = rows.len() / 3;
    let colors = |from: usize| {
        rows[from..from + n]
            .iter()
            .map(|c| number::<i32>(c).map(|c| c as u32))
            .collect::<Result<Vec<_>>>()
    };
    Ok(Palette {
        names: rows[..n].to_vec(),
        light: colors(n)?,
        dark: colors(2 * n)?,
    })
}

fn write(step: &Step, cleared: bool) -> Rows {
    let mut out = vec![Text::new(); step::SIZE];
    let (code, slot) = match step {
        Step::Preview => (kind::PREVIEW, None),
        Step::Cached(slot) => (kind::CACHED, Some(*slot)),
        Step::Keep => (kind::KEEP, None),
        Step::Load {
            extension,
            component,
            path,
        } => {
            out[step::EXTENSION] = number_text(*extension);
            out[step::COMPONENT] = number_text(*component);
            out[step::PATH] = path.clone();
            (kind::LOAD, None)
        }
        Step::Active(slot) => (kind::ACTIVE, Some(*slot)),
        Step::Failed => (kind::FAILED, None),
    };
    out[step::KIND] = number_text(code);
    out[step::CLEARED] = snygg::text(wire::flag::encode(cleared));
    out[step::SLOT] = slot.map(number_text).unwrap_or_default();
    out
}

fn take<'a>(rows: &mut &'a [Text], n: usize) -> Result<&'a [Text]> {
    let (head, rest) = rows
        .split_at_checked(n)
        .ok_or(Error("an index row is cut short"))?;
    *rows = rest;
    Ok(head)
}

fn field(fields: &[Text], at: usize) -> Result<&Text> {
    fields.get(at).ok_or(Error("a row is cut short"))
}

fn utf8(units: &[u16]) -> Result<String> {
    String::from_utf16(units).map_err(|_| Error("not UTF-16"))
}

fn number<T: std::str::FromStr>(units: &[u16]) -> Result<T> {
    utf8(units)?.parse().map_err(|_| Error("not a number"))
}

fn number_text(value: impl ToString) -> Text {
    snygg::text(&value.to_string())
}
