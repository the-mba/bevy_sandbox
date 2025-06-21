use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::transform::components::Transform;

/// The entity that generated this entity
/// Initially, this is the player that fired the bullet.
///
/// This is the source of truth for the relationship,
/// and can be modified directly to change the target.
#[derive(Component)]
#[relationship(relationship_target = Generated)]
pub struct GeneratedBy(Entity);

/// All entities that are targeting this entity.
///
/// This component is updated reactively using the component hooks introduced by deriving
/// the [`Relationship`] trait. We should not modify this component directly,
/// but can safely read its field. In a larger project, we could enforce this through the use of
/// private fields and public getters.
#[derive(Component, Debug)]
#[relationship_target(relationship = GeneratedBy)]
pub struct Generated(Vec<Entity>);

// For the lasers, so we move them with the player
#[derive(Component)]
#[relationship(relationship_target = Anchored)]
pub struct AnchorTo(Entity);

impl AnchorTo {
    pub fn anchor(&self, anchored_transform: &mut Transform, new_transform: &Transform) {
        // Move the anchored entity to the player's position
        anchored_transform.translation = new_transform.translation;
        // Set the rotation to match the player's rotation
        anchored_transform.rotation = new_transform.rotation;
    }
}

#[derive(Component, Debug)]
#[relationship_target(relationship = AnchorTo)]
pub struct Anchored(Vec<Entity>);
