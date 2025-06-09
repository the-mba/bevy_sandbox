use super::player::Generated;
use bevy::{ecs::entity::Entity, prelude::Component};

#[derive(Component)]
pub struct Bullet;

/// The entity that generated this entity
/// Initially, this is the player that fired the bullet.
///
/// This is the source of truth for the relationship,
/// and can be modified directly to change the target.
#[derive(Component)]
#[relationship(relationship_target = Generated)]
pub struct GeneratedBy(Entity);
