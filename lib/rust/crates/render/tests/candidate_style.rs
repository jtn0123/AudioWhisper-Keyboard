use render::{
    raster::Raster,
    scene::{CandidateStyle, Scene},
    style::Style,
    touch::{Motion, Touch},
};

#[test]
fn candidate_press_and_cancel_repaint_the_strip_using_cached_states() {
    let mut scene = Scene {
        width: 80,
        height: 80,
        strip: 20.0,
        background: 0xffffffff,
        candidates: vec![String::new()],
        candidate_styles: vec![std::array::from_fn(|state| CandidateStyle {
            container: Style {
                background: if state == 0 { 0xff123456 } else { 0xffabcdef },
                ..Style::default()
            },
            text: Style::default(),
        })],
        ..Scene::default()
    };
    let full = Scene::default().damage(&scene).unwrap();
    let mut dirty = Raster::new(80, 80, vec![]);
    let mut reference = Raster::new(80, 80, vec![]);
    dirty.draw(&scene, full).unwrap();
    let mut touch = Touch::default();
    for action in [0, 3] {
        let before = scene.clone();
        touch.event(
            &mut scene,
            &Motion {
                action,
                index: 0,
                ids: vec![7],
                times: vec![1, 2],
                coords: vec![10.0, 10.0],
            },
        );
        assert_eq!(
            scene.candidate_pressed,
            if action == 0 { vec![0] } else { vec![] }
        );
        let damage = before.damage_regions(&scene);
        assert_eq!(damage.len(), 1);
        assert_eq!(damage[0].bottom, scene.strip);
        dirty.draw_regions(&scene, &damage).unwrap();
        reference.draw(&scene, full).unwrap();
        assert_eq!(dirty.pixels(), reference.pixels());
        assert_ne!(before, scene);
    }
}
