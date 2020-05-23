pub struct AchievementCondition {
    kind: &'static str,
    value: u64,
}

pub struct Achievement {
    name: String,
    conditions: Vec<AchievementCondition>,
}
