//! Versioned saved-state transport. No preference values or native handles are restored.
use render::settings_ui::{
    App, Modal, backup_saved,
    catalogue::{self, Kind},
    setup, theme_editor,
};
use settings::{Text, text};
/// The saved state's layout; 3 added Back up's and Restore's state, 4 the language screens',
/// 5 the user dictionary screens', 6 the project licence's horizontal scroll, 7 the setup
/// screen's chosen step, 8 the theme editor's draft, 9 its open property editor, 10 its open
/// code editor, with the rule editor's codes kept among its attributes.
const VERSION: &str = "10";
pub fn save(a: &App) -> Vec<Text> {
    // LibrariesContainer keeps its open library, and Jetpref's colour and time preferences their
    // dialogs, in a plain `remember`: recreation closes them.
    let m = a.modal.as_ref().filter(|m| {
        !matches!(
            a.rows()[m.row].kind,
            Kind::Libraries | Kind::Color | Kind::Time
        )
    });
    let e = a.preview_editor();
    vec![
        text(VERSION),
        text(&a.stack.join("|")),
        text(
            &a.scrolls
                .iter()
                .map(f32::to_string)
                .collect::<Vec<_>>()
                .join("|"),
        ),
        text(&m.map_or(-1, |m| m.row as i32).to_string()),
        text(&m.map_or(String::new(), |m| m.selected.clone())),
        text(&m.map_or(String::new(), |m| m.values.join("|"))),
        text(&m.map_or(0.0, |m| m.scroll).to_string()),
        e.text.clone(),
        text(&e.anchor.to_string()),
        text(&e.focus.to_string()),
        text(&e.composing.map_or(-1, |(s, _)| s as i32).to_string()),
        text(&e.composing.map_or(-1, |(_, e)| e as i32).to_string()),
        text(&e.focused.to_string()),
        text(&a.focus.to_string()),
        text(&m.is_none_or(|m| m.enabled).to_string()),
        text(&a.backups_saved()),
        text(&a.languages_saved()),
        text(&a.words_saved()),
        text(&a.project_saved()),
        text(&a.setup_saved()),
        text(&a.theme_editor_saved()),
    ]
}
pub fn restore(a: &mut App, packet: &[Text]) -> Result<(), &'static str> {
    if packet.len() != 21 || packet[0] != text(VERSION) || packet[7].len() > 65_536 {
        return Err("settings saved state version or size");
    }
    let p: Vec<_> = packet.iter().map(|s| String::from_utf16_lossy(s)).collect();
    let number = |i: usize| p[i].parse::<i32>().map_err(|_| "settings saved integer");
    let finite = |s: &str| {
        s.parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .ok_or("settings saved scroll")
    };
    let stack: Vec<_> = p[1].split('|').map(String::from).collect();
    if stack.is_empty()
        || stack.len() > 32
        || stack
            .iter()
            .enumerate()
            .any(|(i, r)| !catalogue::native(r) || (i == 0) != root(r))
    {
        return Err("settings saved route");
    }
    let scrolls = p[2].split('|').map(finite).collect::<Result<Vec<_>, _>>()?;
    if scrolls.len() != stack.len() {
        return Err("settings saved scrolls");
    }
    let rows = catalogue::screen(stack.last().unwrap()).map_or(&[][..], |s| s.rows);
    let row = number(3)?;
    let modal = if row < 0 {
        None
    } else {
        let row = row as usize;
        let r = rows.get(row).ok_or("settings saved modal")?;
        let values: Vec<_> = p[5].split('|').map(String::from).collect();
        let choices: Vec<_> = match r.kind {
            Kind::Choice(c) => c.entries.iter().map(|e| e.0).collect(),
            _ => vec![],
        };
        let values = match r.kind {
            Kind::Choice(_) if choices.contains(&p[4].as_str()) && values.len() == 1 => values,
            Kind::Slider(s)
                if values.len() == if s.second.is_empty() { 1 } else { 2 }
                    && values
                        .iter()
                        .all(|s| s.parse::<f32>().is_ok_and(f32::is_finite)) =>
            {
                values
            }
            Kind::Confirm(..) | Kind::Backup(catalogue::Part::Confirm) if p[5].is_empty() => vec![],
            // The language screens' dialogs keep only `selected`.
            Kind::Pick(_) | Kind::Lang(_) | Kind::ThemeEdit(_) if p[5].is_empty() => vec![],
            _ => return Err("settings saved modal values"),
        };
        Some(Modal {
            row,
            selected: p[4].clone(),
            values,
            scroll: finite(&p[6])?,
            enabled: p[14].parse().map_err(|_| "settings saved switch")?,
        })
    };
    let (anchor, focus) = (number(8)?, number(9)?);
    let (start, end) = (number(10)?, number(11)?);
    if anchor < 0
        || focus < 0
        || anchor as usize > packet[7].len()
        || focus as usize > packet[7].len()
        || !((start == -1 && end == -1)
            || (start >= 0 && end >= start && end as usize <= packet[7].len()))
    {
        return Err("settings saved editor bounds");
    }
    let focused = p[12].parse::<bool>().map_err(|_| "settings saved focus")?;
    let id = number(13)?;
    let horizontal = finite(&p[18])?;
    let setup = App::parse_setup(&p[19])?;
    let (files, backups) = backup_saved::parse_backups(&p[15])?;
    // Checked before anything changes: a languages or words part that fails leaves the state as
    // it was.
    let mut languages = App::default();
    languages.restore_languages(&p[16])?;
    languages.restore_words(&p[17])?;
    languages.restore_theme_editor(&p[20])?;
    a.stack = stack;
    a.scrolls = scrolls;
    a.modal = modal;
    // A saved state keeps the preview field's editor; a row's field restores from its preference.
    a.field = None;
    a.parked = Default::default();
    a.editor.text = packet[7].clone();
    a.editor.anchor = anchor as usize;
    a.editor.focus = focus as usize;
    a.editor.composing = (start >= 0).then_some((start as usize, end as usize));
    a.editor.focused = focused;
    a.focus = id;
    a.fling = None;
    a.down = None;
    a.files = files;
    a.backups = backups;
    a.draft = languages.draft;
    a.languages.search = languages.languages.search;
    a.list = languages.list;
    a.words.draft = languages.words.draft;
    a.words.search = languages.words.search;
    a.words.transfer = languages.words.transfer;
    a.languages_restored();
    a.words_restored();
    a.themes_restored();
    a.project_restored(horizontal);
    a.setup = setup;
    a.theme_editor_restored(languages.theme_editor.restored.take());
    Ok(())
}
/// The routes a back stack starts from and holds nowhere else: home, setup and a theme
/// component's editor.
fn root(route: &str) -> bool {
    route == catalogue::HOME_ROUTE || route == setup::ROUTE || theme_editor::target(route).is_some()
}
