use super::super::super::constants::BULLET_COOLDOWN;
use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Timer;
use bevy::time::TimerMode;

#[derive(Component, Deref, DerefMut)]
pub struct BulletCooldown {
    pub a: Timer,
}

impl Default for BulletCooldown {
    fn default() -> Self {
        BulletCooldown {
            a: Timer::from_seconds(BULLET_COOLDOWN, TimerMode::Once),
        }
    }
}
