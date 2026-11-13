//! Configured subtype order is authoritative; Android supplies localized locale names.
use crate::{
    panels::{Command, Item, Panel, rect},
    theme::{Request, Theme},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Subtypes {
    pub ids: Vec<i64>,
    pub labels: Vec<String>,
    pub selected: i64,
    pub icons: [String; 2],
    pub header: String,
    pub header_height: f32,
    pub row_height: f32,
}

pub fn rows(ids: &[i64], labels: &[String], selected: i64) -> Vec<(i64, String, bool)> {
    ids.iter()
        .zip(labels)
        .map(|(&id, label)| (id, label.clone(), id == selected))
        .collect()
}

pub fn sheet_start(height: f32, header: f32, row: f32, count: usize) -> f32 {
    (height - header - row * count as f32).max(0.0)
}

impl Panel {
    pub fn measure_subtypes(
        &mut self,
        theme: &mut Theme,
        text: &mut crate::text::Text,
        r: &Request,
    ) {
        let Some(s) = &mut self.subtypes else { return };
        let header = theme.style(&r.element("subtype-panel-header", 0));
        let item = theme.style(&r.element("subtype-panel-list-item", 0));
        let icon = theme.style(&r.element("subtype-panel-list-item-icon-leading", 0));
        let label = theme.style(&r.element("subtype-panel-list-item-text", 0));
        s.header_height = text.height(&s.header, header.size, header.mono, header.weight)
            + header.padding[1]
            + header.padding[3];
        s.row_height = s
            .labels
            .iter()
            .map(|v| text.height(v, label.size, label.mono, label.weight))
            .fold(icon.size, f32::max)
            + item.padding[1]
            + item.padding[3];
    }
    pub fn layout_subtypes(&mut self, width: f32, height: f32) {
        let Some(s) = &self.subtypes else { return };
        let start = sheet_start(height, s.header_height, s.row_height, s.ids.len());
        let area = rect(0.0, start, width, height - start);
        self.max_scroll =
            (s.row_height * s.ids.len() as f32 - (height - start - s.header_height)).max(0.0);
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);
        self.items.push(Item::new(
            -70,
            rect(0.0, 0.0, width, start),
            "Dismiss".into(),
            "subtype-scrim",
            Command::Subtype(None),
        ));
        self.items.push(Item::new(
            -71,
            area,
            String::new(),
            "subtype-panel",
            Command::None,
        ));
        self.items.push(Item::new(
            -72,
            rect(0.0, start, width, s.header_height),
            s.header.clone(),
            "subtype-panel-header",
            Command::None,
        ));
        let clip = rect(
            0.0,
            start + s.header_height,
            width,
            height - start - s.header_height,
        );
        if self.max_scroll > 0.0 {
            self.items.push(Item::new(
                -73,
                clip,
                s.header.clone(),
                "panel-scroll",
                Command::Scroll,
            ));
        }
        for (index, (_id, label, selected)) in
            rows(&s.ids, &s.labels, s.selected).into_iter().enumerate()
        {
            let mut item = Item::new(
                700_000 + index as i32,
                rect(
                    0.0,
                    start + s.header_height + index as f32 * s.row_height - self.scroll,
                    width,
                    s.row_height,
                ),
                label,
                "subtype-panel-list-item",
                Command::Subtype(Some(index)),
            );
            item.clip = clip;
            item.selected = selected;
            item.icon = s.icons[usize::from(!selected)].clone();
            self.items.push(item);
        }
    }
    pub fn style_subtypes(&mut self, theme: &mut Theme, r: &Request) {
        if self.kind != 7 {
            return;
        }
        let root = theme.style(&r.element("subtype-panel", 0));
        for i in &mut self.items {
            if i.element == "subtype-panel-header" {
                for style in &mut i.styles {
                    style.shape = root.shape;
                    style.corners = [root.corners[0], root.corners[1], 0.0, 0.0];
                    style.percent = root.percent;
                }
            }
            if i.element == "subtype-scrim" {
                for s in &mut i.styles {
                    s.background = 0x52000000;
                }
            }
            if i.element == "subtype-panel-list-item" {
                let selector = usize::from(self.pressed == Some(i.id));
                i.text = theme.style(&r.element("subtype-panel-list-item-text", selector));
                i.icon_style =
                    theme.style(&r.element("subtype-panel-list-item-icon-leading", selector));
            }
        }
    }
}
