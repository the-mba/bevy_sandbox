use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component, Deref)]
pub struct FloatHeight {
    pub height: f32,
}
