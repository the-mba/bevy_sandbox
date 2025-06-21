use super::components::*;
use super::constants::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub mesh: Mesh2d,
    pub shape: Shape,
    pub material: MeshMaterial2d<ColorMaterial>,
    pub transform: Transform,
    pub player: Player,
    pub bullet_cooldown: BulletCooldown,
    pub laser_cooldown: LaserCooldown,
    pub collider: Collider,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub score: Score,
}

impl PlayerBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        let half_size = PLAYER_SIZE / 2.0;

        let rectangle = Rectangle { half_size };

        Self {
            mesh: Mesh2d(meshes.add(rectangle)),
            shape: Shape::Rectangle(rectangle),
            material: MeshMaterial2d(materials.add(PLAYER_COLOR)),
            transform: Transform {
                translation: PLAYER_STARTING_POSITION.extend(0.0),
                rotation: Quat::from_rotation_z(PLAYER_STARTING_ORIENTATION.angle_to(Vec2::X)),
                ..default()
            },
            player: Player,
            bullet_cooldown: BulletCooldown::default(),
            laser_cooldown: LaserCooldown::default(),
            collider: Collider::cuboid(half_size.x, half_size.y),
            velocity: Velocity::default(),
            acceleration: Acceleration::default(),
            score: Score::default(),
        }
    }
}

#[derive(Bundle)]
pub struct BallBundle {
    pub mesh: Mesh2d,
    pub shape: Shape,
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
        let circle = Circle {
            radius: BALL_DIAMETER / 2.0,
        };

        Self {
            mesh: Mesh2d(meshes.add(circle)),
            shape: Shape::Circle(circle),
            material: MeshMaterial2d(materials.add(BALL_COLOR)),
            transform: Transform::from_translation(starting_position.extend(0.0)),
            ball: Ball,
            velocity: Velocity {
                linvel: Vec2::ZERO,
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct BulletBundle {
    pub mesh: Mesh2d,
    pub shape: Shape,
    pub material: MeshMaterial2d<ColorMaterial>,
    pub transform: Transform,
    pub bullet: Bullet,
    pub velocity: Velocity,
}

impl BulletBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        starting_transform: Transform,
        starting_velocity: Vec3,
    ) -> Self {
        let annulus = Annulus {
            inner_circle: Circle {
                radius: BULLET_RADIUS / 2.0,
            },
            outer_circle: Circle {
                radius: BULLET_RADIUS,
            },
        };

        Self {
            mesh: Mesh2d(meshes.add(annulus)),
            shape: Shape::Annulus(annulus),
            material: MeshMaterial2d(materials.add(BULLET_COLOR)),
            transform: starting_transform,
            bullet: Bullet,
            velocity: Velocity {
                linvel: starting_velocity.truncate(),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct LaserBundle {
    pub mesh: Mesh2d,
    pub shape: Shape,
    pub material: MeshMaterial2d<ColorMaterial>,
    pub transform: Transform,
    pub bullet: Laser,
    pub despawn_timer: DespawnCooldown,
}

impl LaserBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        player_transform: Transform,
    ) -> Self {
        let transform = player_transform
            .with_translation(player_transform.rotation.mul_vec3(Vec3::X) * LASER_LENGTH / 2.0)
            .with_rotation(Quat::default());

        let rectangle = Rectangle {
            half_size: Vec2::new(LASER_LENGTH / 2.0, LASER_WIDTH / 2.0),
        };

        Self {
            mesh: Mesh2d(meshes.add(rectangle)),
            shape: Shape::Rectangle(rectangle),
            material: MeshMaterial2d(materials.add(LASER_COLOR)),
            transform,
            bullet: Laser,
            despawn_timer: DespawnCooldown::new(LASER_LIFE),
        }
    }
}
