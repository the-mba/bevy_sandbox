use super::super::super::constants::BULLET_COOLDOWN;
use bevy::prelude::Component;
use bevy::prelude::Timer;
use bevy::time::TimerMode;
use std::time::Duration;

#[derive(Component)]
pub struct BulletCooldown {
    pub a: Timer,
}

impl BulletCooldown {
    pub fn reset(&mut self) {
        self.a.reset();
    }

    pub fn tick(&mut self, delta: Duration) {
        self.a.tick(delta);
    }

    pub fn elapsed_secs(&self) -> f32 {
        self.a.elapsed_secs()
    }

    pub fn finished(&self) -> bool {
        self.a.finished()
    }
}

impl Default for BulletCooldown {
    fn default() -> Self {
        BulletCooldown {
            a: Timer::from_seconds(BULLET_COOLDOWN, TimerMode::Once),
        }
    }
}
