use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Timer;
use bevy::time::TimerMode;

#[derive(Component, Deref, DerefMut)]
pub struct DespawnCooldown {
    pub a: Timer,
}

impl DespawnCooldown {
    pub fn new(duration: f32) -> Self {
        DespawnCooldown {
            a: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
