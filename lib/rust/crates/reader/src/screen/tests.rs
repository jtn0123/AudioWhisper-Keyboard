use super::*;

const SCREEN: Bounds = bounds(0, 100, 1_080, 1_500);

const fn bounds(left: i32, top: i32, right: i32, bottom: i32) -> Bounds {
    Bounds {
        left,
        top,
        right,
        bottom,
    }
}

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn sideways(rows: i32, columns: i32, up_down: bool, across: bool, name: &str) -> bool {
    is_sideways(rows, columns, up_down, across, &utf16(name))
}

#[test]
fn a_carousel_or_pager_goes_sideways() {
    assert!(sideways(1, 5, true, true, "RecyclerView"));
    assert!(sideways(0, 0, false, true, "RecyclerView"));
    assert!(sideways(
        0,
        0,
        false,
        false,
        "android.widget.HorizontalScrollView"
    ));
    assert!(sideways(
        0,
        0,
        false,
        false,
        "androidx.viewpager2.widget.ViewPager2"
    ));
}

#[test]
fn a_list_going_up_and_down_does_not() {
    assert!(!sideways(0, 0, true, false, "RecyclerView"));
    assert!(!sideways(5, 1, false, false, "ListView"));
    assert!(!sideways(1, 1, false, false, "ScrollView"));
    assert!(!sideways(0, 0, true, true, "ViewPager"));
    // The name must end with it, and case counts.
    assert!(!sideways(0, 0, false, false, "HorizontalScrollViewHolder"));
    assert!(!sideways(0, 0, false, false, "viewpager"));
}

#[test]
fn boxes_overlap_as_rect_intersect_says() {
    let cut = bounds(-20, 50, 500, 400).intersect(SCREEN);
    assert_eq!(cut, Some(bounds(0, 100, 500, 400)));
    assert_eq!(bounds(0, 1_500, 1_080, 1_600).intersect(SCREEN), None);
    let flat = bounds(0, 300, 1_080, 300);
    assert_eq!(flat.intersect(SCREEN), Some(flat));
    assert_eq!(flat.visible_area(SCREEN), 0);
}

#[test]
fn the_area_in_sight_is_what_shows() {
    let lower = bounds(0, 1_200, 1_080, 2_400);
    assert_eq!(lower.visible_area(SCREEN), 1_080 * 300);
    assert_eq!(bounds(0, 1_600, 1_080, 2_400).visible_area(SCREEN), 0);
    assert_eq!(lower.visible_part(SCREEN), bounds(0, 1_200, 1_080, 1_500));
    assert_eq!(bounds(0, 1_600, 1_080, 2_400).visible_part(SCREEN), SCREEN);
}

#[test]
fn a_huge_area_wraps_as_kotlin_does() {
    let all = bounds(i32::MIN, i32::MIN, i32::MAX, i32::MAX);
    // Each side is -1 as an Int: (MAX - MIN) wraps.
    assert_eq!(all.visible_area(all), 1);
}

#[test]
fn the_list_holds_the_most_text() {
    let pieces = [Some(1), Some(1), Some(1), Some(0), None];
    assert_eq!(main_scroller(&pieces, &[1_000, 800]), Some(1));
    assert_eq!(
        main_scroller(&[Some(1), Some(1), Some(0)], &[1_000, 0]),
        Some(0)
    );
    assert_eq!(main_scroller(&[Some(0), Some(1)], &[500, 800]), Some(1));
    assert_eq!(main_scroller(&[Some(0), Some(1)], &[800, 800]), Some(0));
    assert_eq!(main_scroller(&[None], &[]), None);
    assert_eq!(main_scroller(&[], &[0, -5]), None);
}

#[test]
fn text_inside_the_list_is_in_it() {
    let parents = [None, Some(0), Some(1), None];
    assert!(is_within(&parents, Some(2), Some(1)));
    assert!(is_within(&parents, Some(1), Some(1)));
    assert!(!is_within(&parents, Some(0), Some(1)));
    assert!(!is_within(&parents, Some(3), Some(1)));
    assert!(!is_within(&parents, None, Some(1)));
    assert!(!is_within(&parents, Some(2), None));
}

#[test]
fn a_web_pages_piece_is_brought_into_view() {
    assert_eq!(move_into_view(200, 300, 100, 1_500, true), Move::Stay);
    assert_eq!(
        move_into_view(50, 150, 100, 1_500, true),
        Move::ShowOnScreen
    );
    assert_eq!(
        move_into_view(1_400, 1_600, 100, 1_500, false),
        Move::ShowOnScreen
    );
    assert_eq!(
        move_into_view(1_400, 1_600, 100, 1_500, true),
        Move::TurnPages
    );
}

#[test]
fn turning_stops_once_it_fits() {
    assert_eq!(after_turn(50, 150, 100, 1_500), AfterTurn::ShowOnScreen);
    assert_eq!(after_turn(1_200, 1_400, 100, 1_500), AfterTurn::Done);
    assert_eq!(after_turn(1_400, 1_600, 100, 1_500), AfterTurn::TurnAgain);
}

#[test]
fn an_apps_view_well_inside_is_left_alone() {
    assert!(is_inside(200, 300, 100, 1_500));
    assert!(!is_inside(100, 300, 100, 1_500));
    assert!(!is_inside(1_400, 1_500, 100, 1_500));
    assert!(!is_inside(300, 300, 100, 1_500));
    // A height that wraps past Int's end counts as none.
    assert!(!is_inside(-2, i32::MAX, -10, i32::MAX));
}
