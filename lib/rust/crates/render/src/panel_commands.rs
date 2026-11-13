//! Panel state transitions shared by physical and accessibility input.
use crate::{
    panels::{Command, Panel},
    protocol::*,
    touch::Action,
};
impl Panel {
    pub fn activate(&mut self, command: Command) -> Vec<Action> {
        match command {
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
