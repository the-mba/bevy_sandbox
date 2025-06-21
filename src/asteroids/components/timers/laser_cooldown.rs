use super::super::super::constants::LASER_COOLDOWN;
use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Timer;
use bevy::time::TimerMode;

#[derive(Component, Deref, DerefMut)]
pub struct LaserCooldown {
    pub a: Timer,
}

impl Default for LaserCooldown {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(LASER_COOLDOWN, TimerMode::Once);
        timer.set_elapsed(timer.remaining());
        LaserCooldown { a: timer }
    }
}
