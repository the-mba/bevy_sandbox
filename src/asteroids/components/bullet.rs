use bevy::{
    ecs::system::{SystemParam, lifetimeless::Read},
    prelude::*,
};

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct JustFired;

// Define a custom `SystemParam` for our collision hooks.
// It can have read-only access to queries, resources, and other system parameters.
#[derive(SystemParam)]
struct BulletCollisionHooks<'w, 's> {
    one_way_platforms_query: Query<'w, 's, (Read<Player>, Read<GlobalTransform>)>,
    // NOTE: This precludes a `OneWayPlatform` passing through a `OneWayPlatform`.
    other_colliders_query: Query<'w, 's, Option<Read<JustFired>>, With<Collider>>,
}

// Implement the `CollisionHooks` trait for our custom system parameter.
impl CollisionHooks for BulletCollisionHooks<'_, '_> {
    // Below is a description of the logic used for one-way platforms.

    /// Allows entities to pass through [`OneWayPlatform`] entities.
    ///
    /// Passing through is achieved by removing the collisions between the [`OneWayPlatform`]
    /// and the other entity if the entity should pass through.
    /// If a [`PassThroughOneWayPlatform`] is present on the non-platform entity,
    /// the value of the component dictates the pass-through behaviour.
    ///
    /// Entities known to be passing through each [`OneWayPlatform`] are stored in the
    /// [`OneWayPlatform`]. If an entity is known to be passing through a [`OneWayPlatform`],
    /// it is allowed to continue to do so, even if [`PassThroughOneWayPlatform`] has been
    /// set to disallow passing through.
    ///
    /// #### When an entity is known to already be passing through the [`OneWayPlatform`]
    ///
    /// When an entity begins passing through a [`OneWayPlatform`], it is added to the
    /// [`OneWayPlatform`]'s set of active penetrations, and will be allowed to continue
    /// to pass through until it is no longer penetrating the platform.
    ///
    /// #### When an entity is *not* known to be passing through the [`OneWayPlatform`]
    ///
    /// Depending on the setting of [`PassThroughOneWayPlatform`], the entity may be allowed to
    /// pass through.
    ///
    /// If no [`PassThroughOneWayPlatform`] is present, [`PassThroughOneWayPlatform::ByNormal`] is used.
    ///
    /// [`PassThroughOneWayPlatform`] may be in one of three states:
    /// 1. [`PassThroughOneWayPlatform::ByNormal`]
    ///     - This is the default state
    ///     - The entity may be allowed to pass through the [`OneWayPlatform`] depending on the contact normal
    ///         - If all contact normals are in line with the [`OneWayPlatform`]'s local-space up vector,
    ///           the entity is allowed to pass through
    /// 2. [`PassThroughOneWayPlatform::Always`]
    ///     - The entity will always pass through the [`OneWayPlatform`], regardless of contact normal
    ///     - This is useful for allowing an entity to jump down through a platform
    /// 3. [`PassThroughOneWayPlatform::Never`]
    ///     - The entity will never pass through the [`OneWayPlatform`], meaning the platform will act
    ///       as normal hard collision for this entity
    ///
    /// Even if an entity is changed to [`PassThroughOneWayPlatform::Never`], it will be allowed to pass
    /// through a [`OneWayPlatform`] if it is already penetrating the platform. Once it exits the platform,
    /// it will no longer be allowed to pass through.
    fn modify_contacts(&self, contacts: &mut ContactPair, commands: &mut Commands) -> bool {
        // This is the contact modification hook, called after collision detection,
        // but before constraints are created for the solver. Mutable access to the ECS
        // is not allowed, but we can queue commands to perform deferred changes.

        // Differentiate between which normal of the manifold we should use
        enum RelevantNormal {
            Normal1,
            Normal2,
        }

        // First, figure out which entity is the one-way platform, and which is the other.
        // Choose the appropriate normal for pass-through depending on which is which.
        let (platform_entity, one_way_platform, platform_transform, other_entity, relevant_normal) =
            if let Ok((one_way_platform, platform_transform)) =
                self.one_way_platforms_query.get(contacts.collider1)
            {
                (
                    contacts.collider1,
                    one_way_platform,
                    platform_transform,
                    contacts.collider2,
                    RelevantNormal::Normal1,
                )
            } else if let Ok((one_way_platform, platform_transform)) =
                self.one_way_platforms_query.get(contacts.collider2)
            {
                (
                    contacts.collider2,
                    one_way_platform,
                    platform_transform,
                    contacts.collider1,
                    RelevantNormal::Normal2,
                )
            } else {
                // Neither is a one-way-platform, so accept the collision:
                // we're done here.
                return true;
            };

        if one_way_platform.0.contains(&other_entity) {
            let any_penetrating = contacts.manifolds.iter().any(|manifold| {
                manifold
                    .points
                    .iter()
                    .any(|contact| contact.penetration > 0.0)
            });

            if any_penetrating {
                // If we were already allowing a collision for a particular entity,
                // and if it is penetrating us still, continue to allow it to do so.
                return false;
            } else {
                // If it's no longer penetrating us, forget it.
                commands.queue(OneWayPlatformCommand::Remove {
                    platform_entity,
                    entity: other_entity,
                });
            }
        }

        match self.other_colliders_query.get(other_entity) {
            // Pass-through is set to never, so accept the collision.
            Ok(Some(PassThroughOneWayPlatform::Never)) => true,
            // Pass-through is set to always, so always ignore this collision
            // and register it as an entity that's currently penetrating.
            Ok(Some(PassThroughOneWayPlatform::Always)) => {
                commands.queue(OneWayPlatformCommand::Add {
                    platform_entity,
                    entity: other_entity,
                });
                false
            }
            // Default behaviour is "by normal".
            Err(_) | Ok(None) | Ok(Some(PassThroughOneWayPlatform::ByNormal)) => {
                // If all contact normals are in line with the local up vector of this platform,
                // then this collision should occur: the entity is on top of the platform.
                let platform_up = platform_transform.up().truncate().adjust_precision();
                if contacts.manifolds.iter().all(|manifold| {
                    let normal = match relevant_normal {
                        RelevantNormal::Normal1 => manifold.normal,
                        RelevantNormal::Normal2 => -manifold.normal,
                    };

                    normal.length() > Scalar::EPSILON && normal.dot(platform_up) >= 0.5
                }) {
                    true
                } else {
                    // Otherwise, ignore the collision and register
                    // the other entity as one that's currently penetrating.
                    commands.queue(OneWayPlatformCommand::Add {
                        platform_entity,
                        entity: other_entity,
                    });
                    false
                }
            }
        }
    }
}
