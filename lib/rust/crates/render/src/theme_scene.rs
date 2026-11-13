//! Resolve the state cohort at scene/popup updates, before the render worker sees it.
use crate::{
    protocol::*,
    scene::{CandidateStyle, Scene},
    theme::{Request, Theme},
};

impl Theme {
    pub fn apply(
        &mut self,
        scene: &mut Scene,
        mode: &str,
        shift: &str,
        types: &[String],
        candidates: &[i32],
    ) {
        let base = Request {
            element: "window".into(),
            mode: mode.into(),
            shift: shift.into(),
            code: 0,
            key_type: String::new(),
            selector: 0,
            auto_commit: false,
            popup_source: None,
        };
        let trail = self.properties(&base.element("glide-trail", 0));
        scene.trail_color = match snygg::resolve::get(&trail, &snygg::text("foreground")) {
            Some(snygg::value::Value::Color(c)) => crate::style::argb(*c),
            _ => 0xff00ff00,
        };
        scene.background = self.style(&base).background;
        scene.strip_style = self.style(&base.element("smartbar-candidates-row", 0));
        // A transparent strip reveals its smartbar parent's background, then the window.
        let smartbar = self.style(&base.element("smartbar", 0));
        if scene.strip_style.background == 0 {
            scene.strip_style.background = smartbar.background;
        }
        for (i, key) in scene.keys.iter_mut().enumerate() {
            let request = Request {
                code: key.code,
                key_type: types.get(i).cloned().unwrap_or_default(),
                ..base.clone()
            };
            key.styles = std::array::from_fn(|state| {
                self.style(&request.element("key", if !key.enabled { 4 } else { state }))
            });
            key.hints = std::array::from_fn(|state| {
                self.style(&request.element("key-hint", if !key.enabled { 4 } else { state }))
            });
        }
        scene.candidate_styles = scene
            .candidates
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let flags = candidates.get(i).copied().unwrap_or(0);
                let name = if flags & CANDIDATE_CLIPBOARD != 0 {
                    "smartbar-candidate-clip"
                } else {
                    "smartbar-candidate-word"
                };
                let request = Request {
                    auto_commit: flags & CANDIDATE_AUTO_COMMIT != 0,
                    ..base.clone()
                };
                std::array::from_fn(|state| CandidateStyle {
                    container: self.style(&request.element(name, state)),
                    text: self.style(&request.element(&format!("{name}-text"), state)),
                })
            })
            .collect();
    }
    pub fn popup_styles(&mut self, scene: &mut Scene, request: &Request, codes: &[i32]) {
        scene.popup_style = self.style(&request.element("key-popup-box", 0));
        scene.popup_styles = codes
            .iter()
            .map(|&code| {
                let request = Request {
                    code,
                    popup_source: Some(request.code),
                    ..request.clone()
                };
                std::array::from_fn(|state| {
                    self.style(&request.element("key-popup-element", state * 2))
                })
            })
            .collect();
    }
}
