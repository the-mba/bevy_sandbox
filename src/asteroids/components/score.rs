use bevy::prelude::Component;

#[derive(Component)]
pub struct Score {
    pub a: usize,
}

impl Default for Score {
    fn default() -> Self {
        Score { a: 0 }
    }
}
