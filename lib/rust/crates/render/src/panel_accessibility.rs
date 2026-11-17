//! Every visible panel target shares its draw/hit rectangle and action identity.
use crate::{
    accessibility::{Node, Target},
    panels::{Command, Panel},
    protocol::*,
    touch::Action,
};
/// Panel node ids are this plus the item's id, apart from key slots and bar actions.
const PANEL_IDS: i32 = 2_000_000;
pub fn nodes(panel: &Panel) -> Vec<Node> {
    let mut nodes: Vec<Node> = panel
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, i)| {
            if i.label.is_empty() {
                return None;
            }
            let mut content = panel.target_bounds(index);
            if matches!(i.command, Command::None) && i.element != "subtype-panel-header" {
                content = crate::smartbar::inset(content, i.styles[0].padding);
            }
            let r = crate::panel_layout::intersection(content, i.clip);

            // A rounding sliver below the grid is not a visible emoji target.
            if r.is_empty() || r.height() <= 1.0 {
                return None;
            }
            let mut n = Node::new(
                PANEL_IDS + i.id,
                r,
                i.label.clone(),
                "button",
                Target::Panel(index),
            );
            n.enabled = i.enabled && i.element != "subtype-panel-header";
            n.selected = i.selected;
            n.pressed = panel.pressed == Some(i.id);
            if matches!(i.command, Command::None) {
                n.role = "text";
                n.actions.clear();
            }
            if matches!(i.command, Command::Scroll | Command::Gif(11, _)) {
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
            if matches!(i.command, Command::Subtype(Some(_))) {
                n.actions.push(ACCESS_DISMISS);
                n.role = "popup item";
            }
            if matches!(i.command, Command::Clipboard(5, _)) {
                n.actions.push(ACCESS_LONG_CLICK);
            }
            if matches!(
                i.command,
                Command::Clipboard(7..=9, _) | Command::Clipboard(10..=11, _)
            ) {
                n.actions.push(ACCESS_DISMISS);
                n.role = "popup item";
            }
            if !n.enabled {
                n.actions.clear();
            }
            Some(n)
        })
        .collect();
    for (index, i) in panel.items.iter().enumerate() {
        if i.secondary.is_empty() {
            continue;
        }
        let r = crate::smartbar::inset(
            crate::smartbar::inset(i.bounds, i.styles[0].margin),
            i.styles[0].padding,
        );
        let bounds = crate::panel_layout::intersection(
            keys::Rect {
                top: r.top - i.text_offset,
                bottom: r.top - i.text_offset + i.secondary_style.size * 1.2,
                ..r
            },
            i.clip,
        );
        if bounds.is_empty() {
            continue;
        }
        let mut node = Node::new(
            2_500_000 + i.id,
            bounds,
            i.secondary.clone(),
            "text",
            // Never activated (no actions); names its item so it reads with it.
            Target::Panel(index),
        );
        node.enabled = i.enabled;
        node.actions.clear();
        nodes.push(node);
    }
    nodes
}
/// An open popup's first item, which it and everything after it in the panel's items belong to,
/// and the node id of the item that opened it: an emoji's variants or history menu, a clipboard
/// item's menu, or the clear-history dialog. Each is drawn last, over the panel.
pub fn popup(panel: &Panel) -> Option<(usize, Option<i32>)> {
    let (start, opener) = if let Some((index, ..)) = panel.emoji.as_ref().and_then(|e| e.popup) {
        (
            panel
                .items
                .iter()
                .position(|i| i.command == Command::Dismiss)?,
            Command::Emoji(index),
        )
    } else {
        let c = panel.clipboard.as_ref().filter(|c| c.modal())?;
        (
            panel
                .items
                .iter()
                .position(|i| matches!(i.command, Command::Clipboard(10, _)))?,
            c.popup.map_or(Command::Clipboard(2, 0), |index| {
                Command::Clipboard(5, index)
            }),
        )
    };
    let source = panel.items[..start]
        .iter()
        .find(|i| i.command == opener)
        .map(|i| PANEL_IDS + i.id);
    Some((start, source))
}
pub fn activate(panel: &mut Panel, index: usize, action: i32) -> Vec<Action> {
    let Some(i) = panel.items.get(index) else {
        return vec![];
    };
    if matches!(i.command, Command::Gif(11, _)) {
        if let Some(g) = &mut panel.gif {
            g.chip_scroll = crate::gif_panel::scroll_chips(
                g.chip_scroll,
                i.bounds.width(),
                g.chip_widths.iter().sum(),
                action == ACCESS_SCROLL_FORWARD,
            );
        }
        return vec![Action(PANEL_CHANGED, 0, 0)];
    }
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
