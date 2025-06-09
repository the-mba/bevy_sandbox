use super::super::bullet::GeneratedBy;
use bevy::ecs::entity::Entity;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Player;

/// All entities that are targeting this entity.
///
/// This component is updated reactively using the component hooks introduced by deriving
/// the [`Relationship`] trait. We should not modify this component directly,
/// but can safely read its field. In a larger project, we could enforce this through the use of
/// private fields and public getters.
#[derive(Component, Debug)]
#[relationship_target(relationship = GeneratedBy)]
pub struct Generated(Vec<Entity>);
