//! Panel state transitions shared by physical and accessibility input.
use crate::{
    panels::{Command, Panel},
    protocol::*,
    touch::Action,
};
impl Panel {
    pub fn activate(&mut self, command: Command) -> Vec<Action> {
        match command {
            Command::Chrome(op) => vec![Action(IME_CHROME, op, 0)],
            Command::Subtype(id) => vec![Action(
                PANEL_SUBTYPE,
                id.unwrap_or(0) as i32,
                i32::from(id.is_some()),
            )],
            Command::Gif(op, index) => {
                if self.kind == 6 {
                    vec![Action(PANEL_GIF, op, index as i32)]
                } else {
                    self.gif_activate(op, index)
                }
            }
            Command::Clipboard(op, index) => self.clipboard_activate(op, index),
            Command::Mode(kind) => vec![Action(PANEL_MODE, kind, 0)],
            Command::Emoji(index) => {
                if let Some(e) = &mut self.emoji {
                    e.popup = None;
                }
                vec![Action(PANEL_EMOJI, index as i32, 0)]
            }
            Command::Category(category) => {
                if let Some(e) = &mut self.emoji {
                    self.emoji_scrolls[e.category as usize] = self.scroll;
                    e.category = category;
                    e.popup = None;
                    e.refresh();
                    self.scroll = self.emoji_scrolls[category as usize];
                }
                vec![Action(KEY_FEEDBACK, 0, 0), Action(PANEL_CHANGED, 0, 0)]
            }
            Command::History(op, index) => {
                if let Some(e) = &mut self.emoji {
                    e.popup = None;
                }
                vec![Action(PANEL_HISTORY, index as i32, op)]
            }
            Command::Dismiss => {
                if self.kind == 7 {
                    return vec![Action(PANEL_SUBTYPE, 0, 0)];
                }
                if self.clipboard.as_ref().is_some_and(|c| c.modal()) {
                    return self.clipboard_activate(10, 0);
                }
                if let Some(e) = &mut self.emoji {
                    e.popup = None;
                }
                vec![Action(PANEL_CHANGED, 0, 0)]
            }
            _ => vec![],
        }
    }
    pub fn long_press_emoji(&mut self, index: usize) {
        if let Some(e) = &mut self.emoji {
            let history = e.category == 0;
            let pinned = e.shown_pinned.contains(&index);
            if history || !e.variants(index).is_empty() {
                e.popup = Some((index, history, pinned));
            }
        }
    }
}
