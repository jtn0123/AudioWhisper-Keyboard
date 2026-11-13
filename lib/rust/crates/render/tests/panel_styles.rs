use render::theme::{Request, Theme};

#[test]
fn action_label_and_icon_inherit_the_same_disabled_and_pressed_parent_as_compose() {
    let mut theme = Theme::new(
        r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
        "smartbar-action-tile":{"foreground":"rgba(0,0,0,1)"},
        "smartbar-action-tile:disabled":{"foreground":"rgba(180,180,180,1)"},
        "smartbar-action-tile:pressed":{"foreground":"rgba(255,0,0,1)"}}
    "##,
        vec![],
        vec![],
        1.0,
        1.0,
        1.0,
    )
    .unwrap();
    for selector in [0, 1, 4] {
        let request = Request {
            element: "smartbar-action-tile".into(),
            mode: "characters".into(),
            shift: "unshifted".into(),
            code: -31,
            key_type: String::new(),
            selector,
            auto_commit: false,
            popup_source: None,
        };
        let parent = theme.style(&request);
        for child in ["smartbar-action-tile-icon", "smartbar-action-tile-text"] {
            assert_eq!(
                parent.foreground,
                theme.style(&request.element(child, selector)).foreground
            );
        }
    }
}
