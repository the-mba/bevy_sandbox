use bevy::prelude::Component;

// Default must be implemented to define this as a required component for the Wall component below
#[derive(Component, Default)]
pub struct Collider;
