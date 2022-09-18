use bevy::ecs::component::Component;

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
