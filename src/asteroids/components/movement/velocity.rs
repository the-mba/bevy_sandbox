use super::super::super::constants::PLAYER_STARTING_VELOCITY;
use bevy::{
    math::Vec2,
    prelude::{Component, Deref, DerefMut},
};

#[derive(Component, Deref, DerefMut)]
pub struct Velocity {
    pub a: Vec2,
}

impl Default for Velocity {
    fn default() -> Self {
        Velocity {
            a: PLAYER_STARTING_VELOCITY,
        }
    }
}
