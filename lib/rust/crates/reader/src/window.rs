//! Which of the screen's windows a reading reads, as the keyboard's `ReadAloudService` chose it in
//! Kotlin: the app the user is in, and the part of the screen it shows above the keyboard.

/// The app's window among windows that are an application's (`application`) or not: the active
/// one, else the focused one, else the one on top (the first, if several share the top layer).
/// Then the first input method's window, the keyboard, if there is one.
pub fn choose(
    application: &[bool],
    input_method: &[bool],
    active: &[bool],
    focused: &[bool],
    layers: &[i32],
) -> (Option<usize>, Option<usize>) {
    let apps = || (0..application.len()).filter(|&i| application[i]);
    let app = (apps().find(|&i| active[i]))
        .or_else(|| apps().find(|&i| focused[i]))
        .or_else(|| {
            apps().fold(None, |top, i| {
                top.filter(|&t| layers[t] >= layers[i]).or(Some(i))
            })
        });
    (app, (0..input_method.len()).find(|&i| input_method[i]))
}

/// The bottom of the part of the screen an app between `top` and `bottom` shows, with the
/// keyboard's top at `keyboard_top`: cut to the keyboard if it starts inside the app.
pub fn visible_bottom(top: i32, bottom: i32, keyboard_top: i32) -> i32 {
    if keyboard_top > top && keyboard_top < bottom {
        keyboard_top
    } else {
        bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_active_app_then_the_focused_then_the_top_first() {
        let apps = [false, true, true, true];
        let ime = [true, false, false, true];
        let none = [false; 4];
        let layers = [9, 1, 3, 3];
        assert_eq!(
            choose(&apps, &ime, &[true, false, true, true], &none, &layers),
            (Some(2), Some(0))
        );
        assert_eq!(
            choose(&apps, &none, &none, &[false, true, false, true], &layers),
            (Some(1), None)
        );
        assert_eq!(choose(&apps, &none, &none, &none, &layers), (Some(2), None));
        assert_eq!(
            choose(&[false; 2], &[false; 2], &[true; 2], &[true; 2], &[1, 2]),
            (None, None)
        );
    }

    #[test]
    fn the_keyboard_cuts_the_app_only_from_inside() {
        assert_eq!(visible_bottom(100, 2000, 1500), 1500);
        assert_eq!(visible_bottom(100, 2000, 2000), 2000);
        assert_eq!(visible_bottom(100, 2000, 100), 2000);
        assert_eq!(visible_bottom(100, 2000, 2400), 2000);
    }
}
