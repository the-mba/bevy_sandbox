use super::components::*;
use super::constants::*;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub player: Player,
    pub bullet_cooldown: BulletCooldown,
    pub collider: Collider,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub score: Score,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            sprite: Sprite {
                color: PLAYER_COLOR,
                ..default()
            },
            transform: Transform {
                translation: PLAYER_STARTING_POSITION.extend(0.0),
                rotation: Quat::from_rotation_z(PLAYER_STARTING_ORIENTATION.angle_to(Vec2::X)),
                scale: PLAYER_SIZE.extend(1.0),
                ..default()
            },
            player: Player,
            bullet_cooldown: BulletCooldown::default(),
            collider: Collider,
            velocity: Velocity::default(),
            acceleration: Acceleration::default(),
            score: Score::default(),
        }
    }
}

#[derive(Bundle)]
pub struct BallBundle {
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,
    pub transform: Transform,
    pub ball: Ball,
    pub velocity: Velocity,
}

impl BallBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        starting_position: Vec2,
    ) -> Self {
        Self {
            mesh: Mesh2d(meshes.add(Circle::default())),
            material: MeshMaterial2d(materials.add(BALL_COLOR)),
            transform: Transform::from_translation(starting_position.extend(0.0))
                .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.0)),
            ball: Ball,
            velocity: Velocity { a: Vec2::ZERO },
        }
    }
}

#[derive(Bundle)]
pub struct BulletBundle {
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,
    pub transform: Transform,
    pub bullet: Bullet,
    pub velocity: Velocity,
}

impl BulletBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        starting_transform: &Transform,
        starting_velocity: &Vec3,
    ) -> Self {
        Self {
            mesh: Mesh2d(meshes.add(Annulus::default())),
            material: MeshMaterial2d(materials.add(BULLET_COLOR)),
            transform: starting_transform.with_scale(BULLET_SIZE.extend(1.0)),
            bullet: Bullet,
            velocity: Velocity {
                a: starting_velocity.truncate(),
            },
        }
    }
}
