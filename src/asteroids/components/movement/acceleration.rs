use super::super::super::constants::PLAYER_STARTING_ACCELERATION;
use bevy::{
    math::Vec2,
    prelude::{Component, Deref, DerefMut},
};

#[derive(Component, Deref, DerefMut)]
pub struct Acceleration {
    pub a: Vec2,
}

impl Default for Acceleration {
    fn default() -> Self {
        Acceleration {
            a: PLAYER_STARTING_ACCELERATION,
        }
    }
}
