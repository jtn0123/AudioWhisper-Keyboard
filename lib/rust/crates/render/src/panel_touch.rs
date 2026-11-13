//! Panel scrolling, drag markers and press/release state. Android only executes returned commands.
use crate::{
    panels::{Command, Panel},
    protocol::*,
    touch::{Action, Motion},
};
#[derive(Debug, Default)]
pub struct PanelTouch {
    pointer: Option<i32>,
    item: Option<i32>,
    origin: [f32; 2],
    last: [f32; 2],
    deadline: Option<i64>,
    scrolling: bool,
    dragging: bool,
    repeated: bool,
    long_emoji: bool,
    velocity: crate::velocity::Velocity,
    fling: Option<crate::panel_scroll::Fling>,
}
impl PanelTouch {
    pub fn deadline(&self) -> Option<i64> {
        self.deadline
    }
    pub fn cancel(&mut self, panel: &mut Panel) -> Vec<Action> {
        if let Some(e) = &mut panel.editor {
            e.complete();
        }
        self.deadline = None;
        self.fling = None;
        self.pointer = None;
        panel.pressed = None;
        let out = self
            .item
            .take()
            .and_then(|id| panel.items.iter().find(|i| i.id == id))
            .and_then(|i| match i.command {
                Command::Action(id) => Some(Action(PANEL_CANCEL, id, 0)),
                _ => None,
            })
            .into_iter()
            .collect();
        self.repeated = false;
        self.long_emoji = false;
        self.dragging = false;
        self.scrolling = false;
        out
    }
    pub fn tick(&mut self, panel: &mut Panel, now: i64) -> Vec<Action> {
        if self.deadline.is_none_or(|d| now < d) {
            return vec![];
        }
        self.deadline = None;
        if let Some(fling) = &self.fling {
            let value = fling.value(now);
            panel.scroll = value.clamp(0.0, panel.max_scroll);
            if now < fling.start + fling.duration && value > 0.0 && value < panel.max_scroll {
                self.deadline = Some(now + 16_000_000);
            } else {
                self.fling = None;
            }
            return vec![Action(PANEL_CHANGED, 0, 0)];
        }
        if let Some(i) = self
            .item
            .and_then(|id| panel.items.iter().find(|i| i.id == id))
        {
            match i.command {
                Command::Drag(group, index) => {
                    if let Some(e) = &mut panel.editor {
                        e.begin(group, index);
                        self.dragging = e.dragging.is_some();
                    }
                }
                Command::Emoji(index) => {
                    self.long_emoji = true;
                    panel.long_press_emoji(index);
                    return vec![Action(LONG_PRESS_FEEDBACK, 0, 0)];
                }
                Command::Action(id) if crate::editing::repeatable(i.code) => {
                    self.repeated = true;
                    let code = crate::editing::repeat_code(i.code, panel.delete_words);
                    self.deadline =
                        Some(now + crate::editing::repeat_delay(code, panel.repeat_delay));
                    return vec![Action(PANEL_REPEAT, id, code)];
                }
                _ => {}
            }
        }
        vec![]
    }
    pub fn event(&mut self, panel: &mut Panel, event: &Motion, delay: i64) -> Vec<Action> {
        if event.action == 3 || !event.valid() {
            return self.cancel(panel);
        }
        self.velocity.event(event);
        let n = event.coords.len() - event.ids.len() * 2;
        let pointer = event.ids[event.index];
        let pos = [
            event.coords[n + event.index * 2],
            event.coords[n + event.index * 2 + 1],
        ];
        let mut out = vec![];
        match event.action {
            0 => {
                out.extend(self.cancel(panel));
                self.pointer = Some(pointer);
                self.origin = pos;
                self.last = pos;
                panel.drag_position = pos;
                if let Some(item) = panel.hit(pos[0], pos[1]) {
                    self.item = Some(item.id);
                    if matches!(item.command, Command::Emoji(_)) && item.id < 400_000 {
                        out.push(Action(KEY_FEEDBACK, 0, 0));
                    }
                    match item.command {
                        Command::Action(id) => {
                            out.push(Action(PANEL_DOWN, id, pointer));
                            self.deadline = Some(
                                event.times.last().copied().unwrap_or(0)
                                    + crate::editing::hold_delay(item.code, delay / 1_000_000)
                                        * 1_000_000,
                            );
                        }
                        Command::Emoji(..) if item.id >= 400_000 => {}
                        Command::Drag(..) | Command::Emoji(..) => {
                            self.deadline = Some(
                                event.times.last().copied().unwrap_or(0)
                                    + if panel.gesture_delay > 0 {
                                        panel.gesture_delay
                                    } else {
                                        delay
                                    },
                            )
                        }
                        _ => {}
                    }
                    panel.pressed = self.item;
                }
            }
            2 => {
                let Some(p) = self
                    .pointer
                    .and_then(|p| event.ids.iter().position(|id| *id == p))
                else {
                    return out;
                };
                let pos = [event.coords[n + p * 2], event.coords[n + p * 2 + 1]];
                panel.drag_position = pos;
                if self.dragging {
                    if let Some(Command::Drag(group, index)) = panel.drag_at(pos[0], pos[1])
                        && let Some(e) = &mut panel.editor
                    {
                        e.over(group, index);
                    }
                } else {
                    if !self.scrolling
                        && ((pos[0] - self.origin[0]).abs() > 8.0 * panel.density
                            || (pos[1] - self.origin[1]).abs() > 8.0 * panel.density)
                    {
                        self.scrolling = true;
                        self.deadline = None;
                        panel.pressed = None;
                        if let Some(i) = self
                            .item
                            .and_then(|id| panel.items.iter().find(|i| i.id == id))
                            && let Command::Action(id) = i.command
                        {
                            out.push(Action(PANEL_CANCEL, id, pointer));
                        }
                    }
                    if self.scrolling
                        && (panel.kind != 1
                            || (pos[1] - self.origin[1]).abs() >= (pos[0] - self.origin[0]).abs())
                    {
                        panel.scroll =
                            (panel.scroll + self.last[1] - pos[1]).clamp(0.0, panel.max_scroll);
                    }
                }
                self.last = pos;
            }
            1 | 6 if self.pointer == Some(pointer) => {
                if self.long_emoji {
                    // Compose consumes a long press even when the key has no variants.
                } else if self.dragging {
                    if let Some(e) = &mut panel.editor {
                        e.complete();
                    }
                } else if !self.scrolling {
                    let command = panel
                        .hit(pos[0], pos[1])
                        .filter(|i| Some(i.id) == self.item)
                        .map(|i| i.command.clone());
                    match command {
                        Some(Command::Action(id)) => {
                            out.push(Action(
                                if self.repeated {
                                    PANEL_CANCEL
                                } else {
                                    PANEL_UP
                                },
                                id,
                                pointer,
                            ));
                            if panel.kind == 2
                                && panel.entries.iter().find(|e| e.id == id).is_some_and(|e| {
                                    !e.insert_text && crate::editing::closes(e.code)
                                })
                            {
                                out.push(Action(PANEL_MODE, 0, 0));
                            }
                        }
                        Some(command) => out.extend(panel.activate(command)),
                        _ => out.extend(self.cancel(panel)),
                    }
                }
                if self.scrolling
                    && panel.kind == 1
                    && (pos[0] - self.origin[0]).abs() > (pos[1] - self.origin[1]).abs()
                    && (pos[0] - self.origin[0]).abs() > 48.0 * panel.density
                    && let Some(e) = &panel.emoji
                {
                    let category = (e.category + if pos[0] < self.origin[0] { 1 } else { -1 })
                        .clamp(if e.history { 0 } else { 1 }, 9);
                    out.extend(panel.activate(Command::Category(category)));
                }
                let now = *event.times.last().unwrap_or(&0);
                if self.scrolling
                    && (pos[1] - self.origin[1]).abs() >= (pos[0] - self.origin[0]).abs()
                {
                    let velocity = -self.velocity.get(pointer).1;
                    self.fling = crate::panel_scroll::Fling::new(
                        now,
                        panel.scroll,
                        panel.density,
                        velocity.clamp(-8000.0 * panel.density, 8000.0 * panel.density),
                    );
                }
                self.long_emoji = false;
                self.item = None;
                self.pointer = None;
                self.deadline = self.fling.as_ref().map(|_| now + 16_000_000);
                panel.pressed = None;
                self.dragging = false;
            }
            _ => {}
        }
        out
    }
}
