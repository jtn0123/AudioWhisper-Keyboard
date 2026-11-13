use render::arrangement::{Arrangement, Editor};
#[test]
fn moving_to_sticky_displaces_old_sticky_and_cancel_completes() {
    let mut e = Editor::new(Arrangement {
        sticky: 9,
        dynamic: vec![1, 2],
        hidden: vec![],
    });
    e.begin(1, 1);
    e.over(0, 0);
    assert_eq!(
        e.saved(),
        Arrangement {
            sticky: 2,
            dynamic: vec![9, 1],
            hidden: vec![]
        }
    );
}
#[test]
fn overflow_keeps_the_original_compose_last_index_clamp() {
    let a = Arrangement {
        sticky: -1,
        dynamic: vec![1, 2, 3],
        hidden: vec![],
    };
    assert_eq!(a.overflow(0), &[2, 3]);
    assert!(a.overflow(3).is_empty());
}
