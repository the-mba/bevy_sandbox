use bevy::prelude::*;

#[derive(Event, Default)]
pub struct BallCollisionEvent;

#[derive(Event, Default)]
pub struct DestroyedEvent;
