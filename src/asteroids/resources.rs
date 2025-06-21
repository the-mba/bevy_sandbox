use super::constants::BALL_COOLDOWN;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Timer;
use bevy::time::TimerMode;
use bevy::{asset::Handle, audio::AudioSource, prelude::Resource};

#[derive(Resource, Deref)]
pub struct CollisionSound {
    pub a: Handle<AudioSource>,
}

#[derive(Resource, Deref, DerefMut)]
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
