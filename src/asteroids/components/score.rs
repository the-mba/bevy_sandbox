use bevy::prelude::*;
use derive_more::Display;

#[derive(Component, Default, Deref, DerefMut, Display)]
pub struct Score {
    pub a: usize,
}
