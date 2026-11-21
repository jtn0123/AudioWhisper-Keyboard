//! Jetpref invokes a migration for each stored entry, on every import/load. There is no
//! numbered schema dispatcher in this fork: retain every historical key-family rule.
use crate::{
    Result, Text, codec, text,
    value::{Value, object},
};
use keys::{codes, model_data};
use wire::settings::{KEEP, RESET, TRANSFORM, migration};

pub fn predefined(code: i32) -> Result<Value> {
    let key = model_data::PREDEFINED
        .iter()
        .find(|k| k.code == code)
        .ok_or("unknown key")?;
    let types = [
        "character",
        "enter_editing",
        "function",
        "lock",
        "modifier",
        "navigation",
        "system_gui",
        "numeric",
        "placeholder",
        "unspecified",
    ];
    Ok(object([
        ("$", Value::string("text_key")),
        ("type", Value::string(types[key.kind as usize])),
        ("code", Value::Number(i64::from(code))),
        ("label", Value::string(key.label)),
        ("groupId", Value::Number(0)),
        ("popup", Value::Null),
    ]))
}
fn action(code: i32) -> Result<Value> {
    Ok(object([
        ("$", Value::string("insert_key")),
        ("data", predefined(code)?),
    ]))
}
fn same_action(a: &Value, b: &Value) -> bool {
    if a != b {
        return false;
    }
    if a.get("$").ok() == Some(&Value::string("insert_text")) {
        return true;
    }
    let Ok(data) = a.get("data") else {
        return false;
    };
    // Auto/multi keys and PopupSet use JVM object identity. Separate decoded objects
    // cannot deduplicate, even when their JSON contents happen to match.
    data.get("$").ok() == Some(&Value::string("text_key"))
        && data.get("popup").ok() == Some(&Value::Null)
}
fn arrangement(raw: &[u16]) -> Result<Text> {
    let mut value = codec::decode("arrangement", raw, false)?;
    let mut pool = Vec::new();
    for name in ["stickyAction", "dynamicActions", "hiddenActions"] {
        let old = value.get(name)?;
        let items = if name == "stickyAction" {
            if *old == Value::Null {
                vec![]
            } else {
                vec![old.clone()]
            }
        } else {
            old.list()?.to_vec()
        };
        let mut new = Vec::new();
        for mut item in items {
            if item.get("$")? == &Value::string("insert_key")
                && item.get("data")?.get("code").and_then(Value::number).ok()
                    == Some(i64::from(codes::COMPACT_LAYOUT_TO_RIGHT))
            {
                item = action(codes::TOGGLE_COMPACT_LAYOUT)?;
            }
            new.push(item);
        }
        if name == "dynamicActions" {
            pool = new.clone();
        }
        value.set(
            name,
            if name == "stickyAction" {
                new.into_iter().next().unwrap_or(Value::Null)
            } else {
                Value::List(new)
            },
        )?;
    }
    for code in [
        codes::LANGUAGE_SWITCH,
        codes::FORWARD_DELETE,
        codes::IME_HIDE_UI,
    ] {
        let candidate = action(code)?;
        let sticky = value.get("stickyAction")?;
        let hidden = value.get("hiddenActions")?.list()?;
        if !same_action(sticky, &candidate)
            && !pool
                .iter()
                .chain(hidden)
                .any(|a| same_action(a, &candidate))
        {
            pool.push(candidate);
        }
    }
    value.set("dynamicActions", Value::List(pool))?;
    let mut seen = Vec::new();
    for name in ["stickyAction", "dynamicActions", "hiddenActions"] {
        let old = value.get(name)?;
        let items = if name == "stickyAction" {
            if *old == Value::Null {
                vec![]
            } else {
                vec![old.clone()]
            }
        } else {
            old.list()?.to_vec()
        };
        let mut unique = Vec::new();
        for item in items {
            if !seen.iter().any(|a| same_action(a, &item)) {
                seen.push(item.clone());
                unique.push(item);
            }
        }
        value.set(
            name,
            if name == "stickyAction" {
                unique.into_iter().next().unwrap_or(Value::Null)
            } else {
                Value::List(unique)
            },
        )?;
    }
    codec::encode("arrangement", &value)
}
pub fn migrate(kind: &[u16], key: &[u16], raw: &[u16]) -> Result<Vec<Text>> {
    let k = String::from_utf16_lossy(key);
    let mut target = k.as_ref();
    let mut value = raw.to_vec();
    let mut ty = kind.to_vec();
    let reset = k.starts_with("devtools__")
        || matches!(
            k.as_ref(),
            "advanced__settings_language"
                | "other__settings_language"
                | "advanced__show_app_icon"
                | "other__show_app_icon"
                // Rust always draws settings now, so the switch has nothing to choose.
                | "other__rust_settings_prototype"
        )
        || (k == "keyboard__one_handed_mode" && raw == text("OFF"));
    if reset {
        return Ok(plan(RESET, ty, key.to_vec(), value));
    }
    let mut changed = true;
    match k.as_ref() {
        "media__emoji_recently_used" => {
            target = "emoji__history_data";
            value = object([
                ("pinned", Value::List(vec![])),
                (
                    "recent",
                    Value::List(
                        raw.split(|&u| u == 59)
                            .map(|s| Value::String(s.to_vec()))
                            .collect(),
                    ),
                ),
            ])
            .json();
        }
        "media__emoji_recently_used_max_size" => target = "emoji__history_recent_max_size",
        "advanced__settings_theme" => target = "other__settings_theme",
        "advanced__accent_color" => target = "other__accent_color",
        "advanced__incognito_mode" => target = "suggestion__incognito_mode",
        "advanced__force_incognito_mode_from_dynamic" => {
            target = "suggestion__force_incognito_mode_from_dynamic"
        }
        "suggestion__clipboard_content_enabled" => target = "clipboard__suggestion_enabled",
        "suggestion__clipboard_content_timeout" => target = "clipboard__suggestion_timeout",
        "smartbar__action_arrangement" => value = arrangement(raw)?,
        "theme__editor_display_colors_as" => {
            target = "theme__editor_color_representation";
            value = text(if raw == text("RGBA") { "RGB" } else { "HEX" });
        }
        "clipboard__sync_to_floris" | "clipboard__sync_to_system" => {
            ty = text("s");
            value = if raw == text("true") {
                text("ALL_EVENTS")
            } else if raw == text("false") {
                text("NO_EVENTS")
            } else {
                value
            };
        }
        "clipboard__num_history_grid_columns_portrait" => {
            target = "clipboard__history_num_grid_columns_portrait"
        }
        "clipboard__num_history_grid_columns_landscape" => {
            target = "clipboard__history_num_grid_columns_landscape"
        }
        "clipboard__clean_up_old" => target = "clipboard__history_auto_clean_old_enabled",
        "clipboard__clean_up_after" => target = "clipboard__history_auto_clean_old_after",
        "clipboard__auto_clean_sensitive" => {
            target = "clipboard__history_auto_clean_sensitive_enabled"
        }
        "clipboard__auto_clean_sensitive_after" => {
            target = "clipboard__history_auto_clean_sensitive_after"
        }
        "clipboard__limit_history_size" => target = "clipboard__history_size_limit_enabled",
        "clipboard__max_history_size" => target = "clipboard__history_size_limit",
        "clipboard__clear_primary_clip_deletes_last_item" => {
            target = "clipboard__clear_primary_clip_affects_history_if_unpinned"
        }
        _ => changed = false,
    }
    Ok(plan(
        if changed { TRANSFORM } else { KEEP },
        ty,
        text(target),
        value,
    ))
}

/// A `wire::settings::migration` row.
fn plan(action: &str, ty: Text, key: Text, value: Text) -> Vec<Text> {
    let mut row = vec![vec![]; migration::SIZE];
    row[migration::ACTION] = text(action);
    row[migration::TYPE] = ty;
    row[migration::KEY] = key;
    row[migration::VALUE] = value;
    row
}
