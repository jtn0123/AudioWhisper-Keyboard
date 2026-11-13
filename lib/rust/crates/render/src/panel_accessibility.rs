//! Every visible panel target shares its draw/hit rectangle and action identity.
use crate::{
    accessibility::{Node, Target},
    panels::{Command, Panel},
    protocol::*,
    touch::Action,
};
pub fn nodes(panel: &Panel) -> Vec<Node> {
    panel
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, i)| {
            if i.label.is_empty() {
                return None;
            }
            let mut content = crate::smartbar::inset(i.bounds, i.styles[0].margin);
            if matches!(i.command, Command::None) {
                content = crate::smartbar::inset(content, i.styles[0].padding);
            }
            let r = crate::panel_layout::intersection(content, i.clip);

            // A rounding sliver below the grid is not a visible emoji target.
            if r.is_empty() || r.height() <= 1.0 {
                return None;
            }
            let mut n = Node::new(
                2_000_000 + i.id,
                r,
                i.label.clone(),
                "button",
                Target::Panel(index),
            );
            n.enabled = i.enabled;
            n.selected = i.selected;
            n.pressed = panel.pressed == Some(i.id);
            if matches!(i.command, Command::None) {
                n.role = "text";
                n.actions.clear();
            }
            if matches!(i.command, Command::Scroll) {
                n.role = "scroll";
                n.actions = vec![ACCESS_SCROLL_FORWARD, ACCESS_SCROLL_BACKWARD];
            }
            if let Command::Emoji(index) = i.command
                && let Some(e) = &panel.emoji
            {
                if i.id < 400_000 && (e.category == 0 || !e.variants(index).is_empty()) {
                    n.actions.push(ACCESS_LONG_CLICK);
                }
                if e.popup.is_some() && i.id >= 400_000 {
                    n.actions.push(ACCESS_DISMISS);
                    n.role = "popup item";
                }
            }
            if matches!(i.command, Command::Clipboard(5, _)) {
                n.actions.push(ACCESS_LONG_CLICK);
            }
            if !n.enabled {
                n.actions.clear();
            }
            Some(n)
        })
        .collect()
}
pub fn activate(panel: &mut Panel, index: usize, action: i32) -> Vec<Action> {
    let Some(i) = panel.items.get(index) else {
        return vec![];
    };
    if matches!(i.command, Command::Scroll) {
        panel.scroll = (panel.scroll
            + if action == ACCESS_SCROLL_FORWARD {
                panel.row * 3.0
            } else {
                -panel.row * 3.0
            })
        .clamp(0.0, panel.max_scroll);
        return vec![Action(PANEL_CHANGED, 0, 0)];
    }
    if action == ACCESS_LONG_CLICK
        && let Command::Clipboard(5, index) = i.command
    {
        return panel.clipboard_activate(6, index);
    }
    if action == ACCESS_LONG_CLICK
        && let Command::Emoji(index) = i.command
    {
        panel.long_press_emoji(index);
        return vec![Action(PANEL_CHANGED, 0, 0)];
    }
    if action == ACCESS_DISMISS {
        return panel.activate(Command::Dismiss);
    }
    if action != ACCESS_CLICK {
        return vec![];
    }
    match i.command.clone() {
        Command::Action(id) => {
            let mut out = vec![Action(PANEL_DOWN, id, -1), Action(PANEL_UP, id, -1)];
            if panel.kind == 2
                && panel
                    .entries
                    .iter()
                    .find(|e| e.id == id)
                    .is_some_and(|e| !e.insert_text && crate::editing::closes(e.code))
            {
                out.push(Action(PANEL_MODE, 0, 0));
            }
            out
        }
        Command::Mode(kind) => vec![Action(PANEL_MODE, kind, 0)],
        Command::Drag(group, index) => {
            if let Some(e) = &mut panel.editor {
                if e.dragging.is_some() {
                    e.over(group, index);
                    e.complete();
                } else {
                    e.begin(group, index);
                }
            }
            vec![Action(PANEL_CHANGED, 0, 0)]
        }
        Command::None => vec![],
        command => panel.activate(command),
    }
}
