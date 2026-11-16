//! Gesture settings are named at the boundary, rather than relying on SwipeAction enum ordinals.
#[derive(Clone, Debug)]
pub struct GestureConfig {
    pub density: f32,
    pub key_size: f32,
    pub glide: bool,
    pub raw: bool,
    pub shift_pressed: bool,
    pub gif: bool,
    pub show_trail: bool,
    pub trail_duration: i64,
    pub distance: i32,
    pub velocity: i32,
    /// Whether each glide point comes with its time (GLIDE_TIME), which the keyboard's swipe
    /// scoring weighs. Off unless asked for, so a config without it glides as before.
    pub glide_times: bool,
    /// Up, down, left, right, space-up/left/right/hold, delete-left.
    pub actions: [String; 9],
}
impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            density: 1.0,
            key_size: 32.0,
            glide: false,
            raw: false,
            shift_pressed: false,
            gif: false,
            show_trail: true,
            trail_duration: 200_000_000,
            distance: 32,
            velocity: 1900,
            glide_times: false,
            actions: std::array::from_fn(|_| "NO_ACTION".into()),
        }
    }
}
impl GestureConfig {
    pub fn decode(values: &[f32], actions: Vec<String>) -> Option<Self> {
        if ![9, 10, 11].contains(&values.len())
            || actions.len() != 9
            || values.iter().any(|v| !v.is_finite())
            || values[0] <= 0.0
            || values[1] <= 0.0
            || values[7] <= 0.0
            || values[8] < 0.0
        {
            return None;
        }
        Some(Self {
            density: values[0],
            key_size: values[1],
            glide: values[2] != 0.0,
            raw: values[3] != 0.0,
            shift_pressed: values.get(9).is_some_and(|v| *v != 0.0),
            gif: values[4] != 0.0,
            show_trail: values[5] != 0.0,
            trail_duration: (values[6].max(0.0) * 1_000_000.0) as i64,
            distance: values[7] as i32,
            velocity: values[8] as i32,
            glide_times: values.get(10).is_some_and(|v| *v != 0.0),
            actions: actions.try_into().ok()?,
        })
    }
    pub fn action(&self, index: usize) -> &str {
        &self.actions[index]
    }
}
