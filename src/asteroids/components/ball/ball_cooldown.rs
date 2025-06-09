use super::super::super::constants::BALL_COOLDOWN;
use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Timer;
use bevy::time::TimerMode;

#[derive(Component, Deref, DerefMut)]
pub struct BallCooldown {
    pub a: Timer,
}

impl Default for BallCooldown {
    fn default() -> Self {
        BallCooldown {
            a: Timer::from_seconds(BALL_COOLDOWN, TimerMode::Once),
        }
    }
}
