//! Materialized smartbar inputs and measurement, shared by rendering and touch dispatch.
use crate::{
    protocol::*,
    scene::Scene,
    smartbar::{self, Config, Layout},
    style::Style,
    text::Text,
    theme::{Request, Theme},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Action {
    pub code: i32,
    pub enabled: bool,
    pub lit: i32,
    pub level: f32,
    pub warming: bool,
    pub finishing: bool,
    pub icon: String,
    pub label: String,
    pub styles: [Style; 2],
}
#[derive(Clone, Debug, PartialEq)]
pub struct Smartbar {
    pub config: Config,
    pub actions: Vec<Action>,
    pub layout: Layout,
    pub pressed: Vec<usize>,
    pub intrinsic: Vec<f32>,
    pub scroll: f32,
    pub phase: f32,
    pub spacer: Style,
    pub aux: Vec<(String, String)>,
    pub secondary: Vec<Style>,
    pub icons: Vec<Style>,
}
impl Smartbar {
    pub fn new(config: Config, actions: Vec<Action>) -> Self {
        Self {
            config,
            actions,
            layout: Layout::default(),
            pressed: Vec::new(),
            intrinsic: Vec::new(),
            scroll: 0.0,
            phase: 0.0,
            spacer: Style::default(),
            aux: Vec::new(),
            secondary: Vec::new(),
            icons: Vec::new(),
        }
    }
    pub fn relayout(&mut self, width: f32) {
        self.layout = smartbar::layout(&self.config, width, &self.intrinsic, self.scroll);
    }
    pub fn prepare(
        scene: &mut Scene,
        theme: &mut Theme,
        base: &Request,
        text: &mut Text,
        flags: &[i32],
    ) {
        let Some(bar) = &mut scene.smartbar else {
            return;
        };
        bar.spacer = theme.style(&base.element("smartbar-candidate-spacer", 0));
        for (id, action) in bar.actions.iter_mut().enumerate() {
            let name = if id == BAR_TOGGLE_INDEX as usize {
                if bar.config.layout == 3 {
                    "smartbar-extended-actions-toggle"
                } else {
                    "smartbar-shared-actions-toggle"
                }
            } else {
                "smartbar-action-key"
            };
            let request = Request {
                code: action.code,
                ..base.clone()
            };
            action.styles = std::array::from_fn(|state| {
                theme.style(&request.element(name, if action.enabled { state } else { 4 }))
            });
        }
        for action in &mut bar.actions {
            if action.lit != 0 {
                let accent = theme.style(
                    &Request {
                        code: 10,
                        ..base.clone()
                    }
                    .element("key", 0),
                );
                for style in &mut action.styles {
                    style.background = accent.background;
                    style.foreground = accent.foreground;
                    style.shape = snygg::value::ShapeKind::Circle;
                    style.margin = [2.0 * bar.config.density; 4];
                    style.size = 24.0 * bar.config.density;
                }
            }
        }
        bar.secondary.clear();
        bar.icons.clear();
        for i in 0..scene.candidates.len() {
            let auto_commit = flags.get(i).is_some_and(|f| f & CANDIDATE_AUTO_COMMIT != 0);
            let name = if bar.aux.get(i).is_some_and(|a| !a.0.is_empty()) {
                "smartbar-candidate-clip"
            } else {
                "smartbar-candidate-word"
            };
            let request = Request {
                auto_commit,
                ..base.clone()
            };
            bar.secondary
                .push(theme.style(&request.element(&format!("{name}-secondary-text"), 0)));
            bar.icons
                .push(theme.style(&request.element(&format!("{name}-icon"), 0)));
        }
        bar.intrinsic = scene
            .candidates
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = &scene.candidate_styles[i][0];
                let primary = text.measure(s, style.text.size, style.text.mono, style.text.weight);
                let secondary = &bar.secondary[i];
                let second = bar.aux.get(i).map_or(0.0, |a| {
                    text.measure(&a.1, secondary.size, secondary.mono, secondary.weight)
                });
                let icon = if bar.aux.get(i).is_some_and(|a| !a.0.is_empty()) {
                    bar.icons[i].size.round() + bar.icons[i].margin[0] + bar.icons[i].margin[2]
                } else {
                    0.0
                };
                primary.max(second)
                    + icon
                    + style.container.margin[0]
                    + style.container.margin[2]
                    + style.container.padding[0]
                    + style.container.padding[2]
            })
            .collect();
        bar.relayout(scene.width as f32);
    }
}
