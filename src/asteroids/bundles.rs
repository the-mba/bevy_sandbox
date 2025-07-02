use super::character_controller_plugin::*;
use super::components::*;
use super::constants::*;

use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,

    pub character_controller_bundle: CharacterControllerBundle,

    // We do not use rigid_body here, because it is included in the character controller bundle
    // pub rigid_body: RigidBody,
    // We do not use collider here, because it is included in the character controller bundle
    // pub collider: Collider,
    pub transform: Transform,

    pub float_height: FloatHeight,

    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,

    pub bullet_cooldown: BulletCooldown,
    pub laser_cooldown: LaserCooldown,

    pub score: Score,
}

impl PlayerBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        let half_size = PLAYER_SIZE / 2.0;

        let collider = Collider::rectangle(half_size.x, half_size.y);

        // queremos obtener del collider el tamaño, para usarlo en apply_controls, porque tnua controller necesita la altura del personaje

        Self {
            player: Player,

            character_controller_bundle: CharacterControllerBundle::new(collider)
                .with_movement(1250.0),

            // We do not use rigid_body here, because it is included in the character controller bundle
            // rigid_body: RigidBody::Dynamic,
            // We do not use collider here, because it is included in the character controller bundle
            // collider,
            transform: Transform {
                translation: PLAYER_STARTING_POSITION.extend(0.0),
                rotation: Quat::from_rotation_z(PLAYER_STARTING_ORIENTATION.angle_to(Vec2::X)),
                ..default()
            },

            float_height: FloatHeight {
                height: PLAYER_SIZE.y / 2.0 + 0.01, // A small offset to avoid collision issues
            },

            mesh: Mesh2d(meshes.add(Rectangle { half_size })),
            material: MeshMaterial2d(materials.add(PLAYER_COLOR)),

            bullet_cooldown: BulletCooldown::default(),
            laser_cooldown: LaserCooldown::default(),

            score: Score::default(),
        }
    }
}

#[derive(Bundle)]
pub struct BallBundle {
    pub ball: Ball,

    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub transform: Transform,

    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,
}

impl BallBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        starting_position: Vec2,
    ) -> Self {
        let radius = BALL_DIAMETER / 2.0;

        Self {
            ball: Ball,

            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(radius),
            transform: Transform::from_translation(starting_position.extend(0.0)),

            mesh: Mesh2d(meshes.add(Circle { radius })),
            material: MeshMaterial2d(materials.add(BALL_COLOR)),
        }
    }
}

#[derive(Bundle)]
pub struct BulletBundle {
    pub bullet: Bullet,

    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub transform: Transform,
    // We add starting linear velocity explicitly to the bundle
    pub linear_velocity: LinearVelocity,

    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,

    pub colliding_entities: CollidingEntities,
}

impl BulletBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        starting_transform: Transform,
        starting_velocity: Vec2,
    ) -> Self {
        let annulus = Annulus {
            inner_circle: Circle {
                radius: BULLET_INNER_RADIUS,
            },
            outer_circle: Circle {
                radius: BULLET_OUTER_RADIUS,
            },
        };

        Self {
            bullet: Bullet,

            rigid_body: RigidBody::Kinematic,
            collider: Collider::circle(BULLET_OUTER_RADIUS),
            transform: starting_transform,
            linear_velocity: LinearVelocity(starting_velocity),

            mesh: Mesh2d(meshes.add(annulus)),
            material: MeshMaterial2d(materials.add(BULLET_COLOR)),

            colliding_entities: CollidingEntities::default(),
        }
    }
}

#[derive(Bundle)]
pub struct LaserBundle {
    pub bullet: Laser,

    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub transform: Transform,

    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,

    pub despawn_timer: DespawnCooldown,
}

impl LaserBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        player_transform: Transform,
    ) -> Self {
        let half_x = LASER_LENGTH / 2.0;
        let half_y = LASER_WIDTH / 2.0;

        let transform = player_transform
            .with_translation(player_transform.rotation.mul_vec3(Vec3::X) * half_x)
            .with_rotation(Quat::default());

        Self {
            bullet: Laser,

            rigid_body: RigidBody::Dynamic,
            collider: Collider::rectangle(LASER_LENGTH, LASER_WIDTH),
            transform,

            mesh: Mesh2d(meshes.add(Rectangle {
                half_size: Vec2::new(half_x, half_y),
            })),
            material: MeshMaterial2d(materials.add(LASER_COLOR)),

            despawn_timer: DespawnCooldown::new(LASER_LIFE),
        }
    }
}
