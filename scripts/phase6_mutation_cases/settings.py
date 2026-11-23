"""The settings scroller's drag, fling and scrollbar (SettingsDragMatchTest, SettingsScrollbarMatchTest):
the cases the frozen Compose drag and scrollbar references used to catch."""
RENDER = 'lib/rust/crates/render/src/'
EVENTS = RENDER + 'settings_ui/events.rs'
OVERSCROLL = RENDER + 'settings_ui/overscroll.rs'
SCROLLBAR = RENDER + 'settings_ui/scrollbar.rs'
VELOCITY = RENDER + 'compose_velocity.rs'
DECAY = RENDER + 'panel_scroll.rs'
MODULE = ':app'
TARGETS = {
    'drag': ['*SettingsDragMatchTest'],
    'scrollbar': ['*SettingsScrollbarMatchTest'],
}
CASES = [
    # The drag: touch slop, the over-slop first scroll, unmoved events, the tracker's samples.
    ('slop-nine-dp', EVENTS, 'let slop = (8.0 * self.config[2] + 0.5).floor();',
     'let slop = (9.0 * self.config[2] + 0.5).floor();', 'drag'),
    ('slop-not-subtracted', EVENTS, 'delta = total - total.signum() * slop;', 'delta = total;', 'drag'),
    ('sideways-move-ignored', EVENTS, 'if self.dragging && (x, y) == self.last {', 'if self.dragging && y == self.last.1 {',
     'drag'),
    ('down-not-tracked', EVENTS, 'self.velocity.down(down_t.div_euclid(1_000_000), down_y);', 'self.velocity.reset();',
     'drag'),
    ('sample-times-rounded', EVENTS, '.map(|s| (e.times[s + 1].div_euclid(1_000_000), e.coords[s * 2 + 1]))',
     '.map(|s| ((e.times[s + 1] + 500_000).div_euclid(1_000_000), e.coords[s * 2 + 1]))', 'drag'),
    ('history-only-last-sample', VELOCITY, '        for &(ms, position) in samples {',
     '        for &(ms, position) in samples.iter().rev().take(1) {', 'drag'),
    # The velocity fit: the ring, the horizon, the 40 ms stop, the fit's degree and the clamp.
    ('ring-of-nineteen', VELOCITY, 'const HISTORY: usize = 20;', 'const HISTORY: usize = 19;', 'drag'),
    ('horizon-80-ms', VELOCITY, 'const HORIZON_MS: f32 = 100.0;', 'const HORIZON_MS: f32 = 80.0;', 'drag'),
    ('up-at-40-ms-stops', VELOCITY, 'if ms - self.last_change > STOPPED_MS as i64 {',
     'if ms - self.last_change >= STOPPED_MS as i64 {', 'drag'),
    ('linear-fit', VELOCITY, 'poly_fit(&times[..count], &points[..count], 2)[1] * 1000.0',
     'poly_fit(&times[..count], &points[..count], 1)[1] * 1000.0', 'drag'),
    ('maximum-halved', EVENTS, 'let maximum = (8000.0 * self.config[2] + 0.5).floor();',
     'let maximum = (4000.0 * self.config[2] + 0.5).floor();', 'drag'),
    # The fling's decay and the stretch overscroll at the ends.
    ('friction-doubled', DECAY, 'let coefficient = 0.015 * (386.0878 * density * 160.0 * 0.84);',
     'let coefficient = 0.03 * (386.0878 * density * 160.0 * 0.84);', 'drag'),
    ('inflexion-moved', DECAY, 'let logarithm = ((0.35 * velocity.abs()) as f64',
     'let logarithm = ((0.3 * velocity.abs()) as f64', 'drag'),
    ('pull-threshold-raised', OVERSCROLL, 'if over.abs() > 0.5 {', 'if over.abs() > 50.0 {', 'drag'),
    ('slack-not-relaxed', OVERSCROLL, 'let used = moved.abs().min(self.stretch.slack);', 'let used = 0.0_f32;', 'drag'),
    # The scrollbar: the 1850 ms hold, the 950 ms fade, its easing, and the hold after a scroll.
    ('hold-shortened', SCROLLBAR, 'let fade_start = (self.scrollbar_start[slot].saturating_add(1_850_000_000))',
     'let fade_start = (self.scrollbar_start[slot].saturating_add(1_800_000_000))', 'scrollbar'),
    ('fade-lengthened', SCROLLBAR, 'as f32 / 950_000_000.0).clamp(0.0, 1.0);', 'as f32 / 1_000_000_000.0).clamp(0.0, 1.0);',
     'scrollbar'),
    ('easing-x-control', SCROLLBAR, '3.0 * (1.0 - t) * t * t * 0.82 + t * t * t;', '3.0 * (1.0 - t) * t * t * 0.8 + t * t * t;',
     'scrollbar'),
    ('easing-y-control', SCROLLBAR, '(3.0 * (1.0 - t) * t * t * (-0.13) + t * t * t)',
     '(3.0 * (1.0 - t) * t * t * (-0.1) + t * t * t)', 'scrollbar'),
    ('solver-coarse', SCROLLBAR, 'for _ in 0..24 {', 'for _ in 0..6 {', 'scrollbar'),
    ('scroll-does-not-hold', SCROLLBAR, '            .max(self.scrollbar_last[slot]);\n',
     '            .min(self.scrollbar_last[slot]);\n', 'scrollbar'),
]
