//! Compose's Lsq2 tracker window rules; SettingsDragMatchTest checks the fit against the real one.
use render::compose_velocity::{ComposeVelocity, poly_fit};

#[test]
fn quadratic_fit_recovers_exact_polynomials() {
    let x = [0.0, -8.0, -16.0, -24.0];
    let y: Vec<f32> = x.iter().map(|t| 5.0 + 2.0 * t + 0.25 * t * t).collect();
    let c = poly_fit(&x, &y, 2);
    assert!((c[0] - 5.0).abs() < 1e-3 && (c[1] - 2.0).abs() < 1e-4 && (c[2] - 0.25).abs() < 1e-5);
}

#[test]
fn needs_three_samples_inside_the_horizon_without_a_40ms_gap() {
    let mut v = ComposeVelocity::default();
    v.down(0, 100.0);
    v.change(&[(8, 90.0)]);
    assert_eq!(v.velocity(1e6), 0.0, "two samples");
    v.change(&[(16, 80.0)]);
    assert!(
        (v.velocity(1e6) + 1250.0).abs() < 0.5,
        "{}",
        v.velocity(1e6)
    );
    assert_eq!(v.velocity(1000.0), -1000.0, "clamped to the maximum");
    // A 41 ms gap cuts the window at the newest samples.
    v.change(&[(57, 70.0), (65, 60.0)]);
    assert_eq!(v.velocity(1e6), 0.0);
    // Samples older than 100 ms fall out of the fit.
    let mut v = ComposeVelocity::default();
    v.down(0, 0.0);
    v.change(
        &(1..=30)
            .map(|i| (i * 10, (i * i) as f32))
            .collect::<Vec<_>>(),
    );
    let recent = poly_fit(
        &(0..=10).map(|i| -(i as f32) * 10.0).collect::<Vec<_>>(),
        &(0..=10)
            .map(|i| ((30 - i) * (30 - i)) as f32)
            .collect::<Vec<_>>(),
        2,
    )[1] * 1000.0;
    assert!((v.velocity(1e6) - recent).abs() < 1e-3);
}

#[test]
fn an_up_after_a_pause_over_40ms_stops_the_fling() {
    let mut v = ComposeVelocity::default();
    v.down(0, 0.0);
    v.change(&[(10, 10.0), (20, 20.0), (30, 30.0)]);
    let mut kept = v.clone();
    kept.up(70);
    assert!(kept.velocity(1e6) > 900.0);
    v.up(71);
    assert_eq!(v.velocity(1e6), 0.0);
}
