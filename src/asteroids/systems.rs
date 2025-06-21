use super::bundles::*;
use super::components::*;
use super::constants::*;
use super::events::*;
use super::resources::*;
use bevy::math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::PrimaryWindow;
use bevy::window::WindowRef;
use bevy_rapier2d::prelude::*;

// Add the game's entities to our world
pub fn setup(
    mut window: Single<&mut Window>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Set up the window
    window.set_maximized(true);

    // Camera
    commands.spawn(Camera2d);

    // Player
    commands.spawn(PlayerBundle::new(&mut meshes, &mut materials));

    // Sound
    commands.insert_resource(CollisionSound {
        a: asset_server.load("sounds/breakout_collision.ogg"),
    });

    // Scoreboard
    commands.spawn((
        Text::new("Score: "),
        TextFont {
            font_size: SCOREBOARD_FONT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        ScoreboardUi,
        Node {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        },
        children![(
            TextSpan::new("0"),
            TextFont {
                font_size: SCOREBOARD_FONT_SIZE,
                ..default()
            },
            TextColor(SCORE_COLOR),
        )],
    ));

    /* Create the ground. */
    commands
        .spawn(Collider::cuboid(500.0, 50.0))
        .insert(Transform::from_xyz(0.0, -100.0, 0.0));

    /* Create the bouncing ball. */
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(50.0))
        .insert(Restitution::coefficient(0.7))
        .insert(Transform::from_xyz(0.0, 400.0, 0.0));
}

trait IfZeroSet {
    fn if_zero_set(&mut self, value: f32);
}

impl IfZeroSet for f32 {
    fn if_zero_set(&mut self, value: f32) {
        if *self == 0.0 {
            *self = value;
        }
    }
}

pub fn calculate_acceleration(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_gamepad: Query<&Gamepad>,
    mut query: Query<(Entity, &Velocity, &mut Acceleration, Option<&Player>)>,
) {
    for (entity, velocity, mut acceleration, maybe_player) in &mut query {
        if maybe_player.is_some() {
            acceleration.x = 0.0;
            acceleration.y = 0.0;

            let mut is_braking = false;

            let key_left = keyboard_input.pressed(KeyCode::ArrowLeft);
            let key_right = keyboard_input.pressed(KeyCode::ArrowRight);
            let key_up = keyboard_input.pressed(KeyCode::ArrowUp);
            let key_down = keyboard_input.pressed(KeyCode::ArrowDown);

            if key_left && !key_right {
                acceleration.x.if_zero_set(-PLAYER_ACCELERATION);
            } else if key_right && !key_left {
                acceleration.x.if_zero_set(PLAYER_ACCELERATION);
            }

            if key_down && !key_up {
                acceleration.y.if_zero_set(-PLAYER_ACCELERATION);
            } else if key_up && !key_down {
                acceleration.y.if_zero_set(PLAYER_ACCELERATION);
            }

            for gamepad in q_gamepad {
                if let Some(left_stick_x) = gamepad.get(GamepadAxis::LeftStickX) {
                    if left_stick_x.abs() > 0.01 {
                        // If the left stick is moved, set the acceleration
                        acceleration
                            .x
                            .if_zero_set(left_stick_x * PLAYER_ACCELERATION);
                    }
                }

                if let Some(left_stick_y) = gamepad.get(GamepadAxis::LeftStickY) {
                    if left_stick_y.abs() > 0.01 {
                        // If the left stick is moved, set the acceleration
                        acceleration
                            .y
                            .if_zero_set(left_stick_y * PLAYER_ACCELERATION);
                    }
                }
            }

            // BRAKING
            if acceleration.x * velocity.linvel.x < 0.0 {
                // If the acceleration is in the opposite direction of the velocity, apply braking
                let acc_x = acceleration.x * PLAYER_BRAKING_MULTIPLIER;
                acceleration.x.if_zero_set(acc_x);

                is_braking = true;
            }

            if acceleration.y * velocity.linvel.y < 0.0 {
                // If the acceleration is in the opposite direction of the velocity, apply braking
                let acc_y = acceleration.y * PLAYER_BRAKING_MULTIPLIER;
                acceleration.y.if_zero_set(acc_y);

                is_braking = true;
            }

            // If no input, apply braking
            if acceleration.x == 0.0 && acceleration.y == 0.0 {
                if velocity.linvel.x != 0.0 {
                    acceleration.x = -velocity.linvel.x.signum()
                        * PLAYER_ACCELERATION
                        * PLAYER_BRAKING_MULTIPLIER;

                    is_braking = true;
                }
                if velocity.linvel.y != 0.0 {
                    acceleration.y = -velocity.linvel.y.signum()
                        * PLAYER_ACCELERATION
                        * PLAYER_BRAKING_MULTIPLIER;

                    is_braking = true;
                }
            }

            if is_braking {
                commands.entity(entity).insert(IsBraking);
            } else {
                commands.entity(entity).remove::<IsBraking>();
            }

            // Normalize the acceleration vector to ensure consistent acceleration magnitude
            acceleration.a = acceleration.normalize_or_zero() * PLAYER_ACCELERATION;
        }
    }
}

pub fn calculate_player_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Acceleration, Option<&IsBraking>), With<Player>>,
) {
    for (mut velocity, acceleration, maybe_is_braking) in &mut query {
        let vel_x_sign = velocity.linvel.x.signum();
        let vel_y_sign = velocity.linvel.y.signum();

        // Update the player's velocity based on the acceleration
        velocity.linvel.x += acceleration.x * time.delta_secs();
        velocity.linvel.y += acceleration.y * time.delta_secs();

        if maybe_is_braking.is_some() {
            // If the velocity changed direction, reset it to 0
            if vel_x_sign != velocity.linvel.x.signum() {
                velocity.linvel.x = 0.0;
            }
            if vel_y_sign != velocity.linvel.y.signum() {
                velocity.linvel.y = 0.0;
            }
        }

        // Limit the player's speed to PLAYER_SPEED
        let speed = velocity.linvel.length();
        if speed > PLAYER_SPEED {
            velocity.linvel = velocity.linvel.normalize_or_zero() * PLAYER_SPEED;
        }
    }
}

pub fn calculate_ball_velocity(
    mut q_balls: Query<(&mut Velocity, &Transform), (With<Ball>, Without<Player>)>,
    q_players: Query<&Transform, With<Player>>,
) {
    for player_transform in &q_players {
        for (mut ball_velocity, ball_transform) in &mut q_balls {
            let direction = player_transform.translation - ball_transform.translation;
            ball_velocity.linvel = direction.truncate();
        }
    }
}

// * Important: must be executed after methods that affect the velocity of objects
pub fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.linvel.x * time.delta_secs();
        transform.translation.y += velocity.linvel.y * time.delta_secs();
    }
}

pub fn calculate_player_orientation(
    q_gamepad: Query<&Gamepad>,
    mut q_player: Query<&mut Transform, With<Player>>,
) {
    for mut transform in &mut q_player {
        for gamepad in &q_gamepad {
            let right_stick = gamepad.right_stick();
            if right_stick.length() > 0.01 {
                // Calculate the angle based on the left stick vector
                let angle = right_stick.y.atan2(right_stick.x);
                transform.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}

pub fn spawn_attacks(
    time: Res<Time>,
    mut commands: Commands,
    r_keyboard_input: Res<ButtonInput<KeyCode>>,
    q_gamepad: Query<&Gamepad>,
    mut r_mesh: ResMut<Assets<Mesh>>,
    mut r_material: ResMut<Assets<ColorMaterial>>,
    mut q_player: Query<
        (Entity, &Transform, &mut BulletCooldown, &mut LaserCooldown),
        With<Player>,
    >,
) {
    for (player_entity, player_transform, mut bullet_cooldown, mut laser_cooldown) in &mut q_player
    {
        bullet_cooldown.tick(time.delta());

        let gamepad_fire_bullet = q_gamepad
            .iter()
            .any(|gamepad| gamepad.pressed(GAMEPAD_BULLET_BUTTON));

        if gamepad_fire_bullet || r_keyboard_input.pressed(KEYBOARD_BULLET_BUTTON) {
            if bullet_cooldown.finished() {
                // Spawn a bullet at the paddle's position
                let bullet_position = player_transform;
                let bullet_velocity = player_transform.rotation.mul_vec3(Vec3::X) * BULLET_SPEED;

                commands.spawn(BulletBundle::new(
                    &mut r_mesh,
                    &mut r_material,
                    *bullet_position,
                    bullet_velocity,
                ));

                // Reset the bullet cooldown timer
                bullet_cooldown.reset();
            }
        }

        laser_cooldown.tick(time.delta());

        let gamepad_fire_laser = q_gamepad
            .iter()
            .any(|gamepad| gamepad.pressed(GAMEPAD_LASER_BUTTON));

        if gamepad_fire_laser || r_keyboard_input.pressed(KEYBOARD_LASER_BUTTON) {
            if laser_cooldown.finished() {
                let laser = commands
                    .spawn(LaserBundle::new(
                        &mut r_mesh,
                        &mut r_material,
                        *player_transform,
                    ))
                    .id();

                commands.entity(player_entity).add_child(laser);

                // Reset the laser cooldown timer
                laser_cooldown.reset();
            }
        }
    }
}

// TODO: arreglar las colisiones, estaban basadas en los scale, pero ahora los scale son (1, 1) y bevy cree que tienen ese tamaño. Cambiarlo a que use el tamaño del mesh

pub fn spawn_balls(
    time: Res<Time>,
    mut ball_cooldown: ResMut<BallCooldown>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q_window: Query<&Window>,
) {
    ball_cooldown.tick(time.delta());

    if ball_cooldown.finished() {
        for window in &q_window {
            // Spawn a ball outside the window at a random position
            let ball_x = match getrandom::u32() {
                Ok(x) => (x as f32 / u32::MAX as f32 - 0.5) * window.width(),
                Err(_) => {
                    info!("Failed to get random x coordinate for ball");
                    return;
                }
            };
            let ball_y = match getrandom::u32() {
                Ok(y) => (y as f32 / u32::MAX as f32 - 0.5) * window.height(),
                Err(_) => {
                    info!("Failed to get random y coordinate for ball");
                    return;
                }
            };

            let ball_position = Vec2::new(ball_x, ball_y);

            // Check if the ball is inside the window bounds
            let inside_window = false; /* ball_position.x >= window.width()
            || ball_position.x <= -window.width()
            || ball_position.y >= window.height()
            || ball_position.y <= -window.height();*/

            if !inside_window {
                // Spawn the ball only if it's outside the window
                commands.spawn(BallBundle::new(&mut meshes, &mut materials, ball_position));
            }

            // Reset the bullet cooldown timer
            ball_cooldown.reset();
        }
    }
}

pub fn despawn(
    time: Res<Time>,
    mut commands: Commands,
    mut q_despawn: Query<(Entity, &mut DespawnCooldown, Option<&ChildOf>)>,
) {
    for (despawn_entity, mut despawn_cooldown, maybe_child_of) in &mut q_despawn {
        despawn_cooldown.tick(time.delta());

        if despawn_cooldown.finished() {
            if let Some(child_of) = maybe_child_of {
                // If the entity has a parent, we need to remove it from the parent's children
                commands
                    .entity(child_of.parent())
                    .remove_children(&[despawn_entity]);
            }
            commands.entity(despawn_entity).despawn();
        }
    }
}

pub fn update_scoreboard(
    score_query: Single<(Entity, &Score), (With<ScoreboardUi>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    let (score_board_ui_entity, score) = *score_query;
    *writer.text(score_board_ui_entity, 1) = score.a.to_string();
}

pub fn window_collision(
    q_window_primary: Query<&Window, With<PrimaryWindow>>,
    q_window: Query<&Window, Without<PrimaryWindow>>,
    camera_query: Query<(&Camera, &Transform), With<Camera>>,
    mut collider_query: Query<(&mut Transform, &mut Velocity), (With<Collider>, Without<Camera>)>,
) {
    for (camera, camera_transform) in &camera_query {
        let window: &Window = match camera.target {
            // the camera is rendering to the primary window
            RenderTarget::Window(WindowRef::Primary) => q_window_primary.single().unwrap(),
            // the camera is rendering to some other window
            RenderTarget::Window(WindowRef::Entity(e_window)) => q_window.get(e_window).unwrap(),
            // the camera is rendering to something else (like a texture), not a window
            _ => {
                // skip this camera
                continue;
            }
        };

        for (mut collider_transform, mut collider_velocity) in &mut collider_query {
            let collider_pos = collider_transform.translation;
            let collider_size = collider_transform.scale.truncate() / 2.0;

            let camera_pos = camera_transform.translation;

            let half_width = window.width() / 2.0;
            let half_height = window.height() / 2.0;

            let camera_left = camera_pos.x - half_width + collider_size.x;
            let camera_right = camera_pos.x + half_width - collider_size.x;
            let camera_top = camera_pos.y + half_height - collider_size.y;
            let camera_bottom = camera_pos.y - half_height + collider_size.y;

            let in_camera = collider_pos.x >= camera_left
                && collider_pos.x <= camera_right
                && collider_pos.y >= camera_bottom
                && collider_pos.y <= camera_top;

            if !in_camera {
                collider_transform.translation = collider_transform.translation.clamp(
                    Vec3::new(camera_left, camera_bottom, 0.0),
                    Vec3::new(camera_right, camera_top, 0.0),
                );

                // If the collider is outside the camera's viewport, reset its velocity
                collider_velocity.linvel.x = 0.0;
                collider_velocity.linvel.y = 0.0;
            }
        }
    }
}

pub fn check_for_ball_collisions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q_ball: Query<(Entity, &Transform), (With<Ball>, Without<Player>)>,
    q_player: Query<(Entity, &Transform), (With<Player>, Without<Ball>)>,
    mut ball_collision_events: EventWriter<BallCollisionEvent>,
) {
    let mut we_deh = false;

    for (_, ball_transform) in q_ball {
        for (_, player_transform) in q_player {
            let collision = collision(
                BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
                ball_transform.translation.truncate(),
                &Aabb2d::new(
                    player_transform.translation.truncate(),
                    player_transform.scale.truncate() / 2.,
                ),
            );

            if collision.is_some() {
                // Writes a collision event so that other systems can react to the collision
                ball_collision_events.write_default();

                we_deh = true;
            }
        }
    }

    if we_deh {
        for (ball_entity, _) in q_ball {
            commands.entity(ball_entity).despawn();
        }

        for (player_entity, _) in q_player {
            commands.entity(player_entity).despawn();
        }

        commands.spawn(PlayerBundle::new(&mut meshes, &mut materials));
    }
}

// * It must execute before normal collision detection, so that the bullets can be despawned
pub fn check_for_bullet_collisions(
    mut commands: Commands,
    q_bullet: Query<(Entity, &Transform), With<Bullet>>,
    q_collider: Query<(Entity, &Transform), (With<Ball>, Without<Bullet>, Without<Player>)>,
    mut bullet_collision_events: EventWriter<BallCollisionEvent>,
) {
    for (bullet_entity, bullet_transform) in q_bullet {
        for (ball_entity, collider_transform) in q_collider {
            // TODO: add the boundingcircle (or alternative) to the bundles
            let bullet_object = BoundingCircle::new(
                bullet_transform.translation.truncate(),
                bullet_transform.scale.x / 2.,
            );
            let bounding_box = Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.,
            );

            if bullet_object.intersects(&bounding_box) {
                // Writes a collision event so that other systems can react to the collision
                bullet_collision_events.write_default();

                commands.entity(bullet_entity).despawn();
                commands.entity(ball_entity).despawn();
            }
        }
    }
}

// * It must execute before normal collision detection, so that the bullets can be despawned
pub fn check_for_laser_collisions(
    mut commands: Commands,
    q_laser: Query<(&Transform, &Shape), With<Laser>>,
    q_collider: Query<(Entity, &Transform), (With<Ball>, Without<Laser>, Without<Player>)>,
    mut bullet_collision_events: EventWriter<BallCollisionEvent>,
) {
    for (laser_transform, laser_shape) in q_laser {
        if let Shape::Rectangle(shape_rectangle) = laser_shape {
            for (ball_entity, collider_transform) in q_collider {
                let laser_object = Aabb2d::new(
                    laser_transform.translation.truncate(),
                    shape_rectangle.half_size,
                );

                let bounding_box = Aabb2d::new(
                    collider_transform.translation.truncate(),
                    collider_transform.scale.truncate() / 2.,
                );

                if laser_object.intersects(&bounding_box) {
                    // Writes a collision event so that other systems can react to the collision
                    bullet_collision_events.write_default();

                    commands.entity(ball_entity).despawn();
                }
            }
        }
    }
}

// Returns `Some` if `ball` collides with `bounding_box`.
// The returned `Collision` is the side of `bounding_box` that `ball` hit.
fn collision(
    object: impl IntersectsVolume<Aabb2d>,
    object_center: Vec2,
    bounding_box: &Aabb2d,
) -> Option<Collision> {
    if !object.intersects(bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(object_center);
    let offset = object_center - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

pub fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<BallCollisionEvent>,
    sound: Res<CollisionSound>,
) {
    // Play a sound once per frame if a collision occurred.
    if !collision_events.is_empty() {
        // This prevents events staying active on the next frame.
        collision_events.clear();
        commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}
