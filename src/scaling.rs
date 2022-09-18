use

pub fn change_scaling(input: Res<Input<KeyCode>>, mut ui_scale: ResMut<TargetScale>) {
    if input.just_pressed(KeyCode::Up) {
        let scale = (ui_scale.target_scale * 2.0).min(8.);
        ui_scale.set_scale(scale);
        info!("Scaling up! Scale: {}", ui_scale.target_scale);
    }
    if input.just_pressed(KeyCode::Down) {
        let scale = (ui_scale.target_scale / 2.0).max(1. / 8.);
        ui_scale.set_scale(scale);
        info!("Scaling down! Scale: {}", ui_scale.target_scale);
    }
}

#[derive(Resource)]
pub struct TargetScale {
    start_scale: f64,
    target_scale: f64,
    target_time: Timer,
}

impl TargetScale {
    fn set_scale(&mut self, scale: f64) {
        self.start_scale = self.current_scale();
        self.target_scale = scale;
        self.target_time.reset();
    }

    fn current_scale(&self) -> f64 {
        let completion = self.target_time.percent();
        let multiplier = ease_in_expo(completion as f64);
        self.start_scale + (self.target_scale - self.start_scale) * multiplier
    }

    fn tick(&mut self, delta: Duration) -> &Self {
        self.target_time.tick(delta);
        self
    }

    fn already_completed(&self) -> bool {
        self.target_time.finished() && !self.target_time.just_finished()
    }
}
