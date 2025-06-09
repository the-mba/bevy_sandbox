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

// Add the game's entities to our world
pub fn setup(
    mut window: Single<&mut Window>,
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Set up the window
    window.set_maximized(true);

    // Camera
    commands.spawn(Camera2d);

    // Player
    commands.spawn(PlayerBundle::default());

    // TODO: extract method to create balls in Update
    // commands.spawn(BallBundle::new(&mut meshes, &mut materials, &ball_speed));

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound {
        a: ball_collision_sound,
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
}

pub fn calculate_player_acceleration(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Acceleration, Option<&Player>)>,
) {
    for (mut velocity, mut acceleration, maybe_player) in &mut query {
        if maybe_player.is_some() {
            acceleration.x = 0.0;
            acceleration.y = 0.0;

            let key_left = keyboard_input.pressed(KeyCode::ArrowLeft);
            let key_right = keyboard_input.pressed(KeyCode::ArrowRight);
            let key_up = keyboard_input.pressed(KeyCode::ArrowUp);
            let key_down = keyboard_input.pressed(KeyCode::ArrowDown);

            if key_left && !key_right {
                acceleration.x = -PLAYER_ACCELERATION;
                velocity.x = velocity.x.min(0.0);
            } else if key_right && !key_left {
                acceleration.x = PLAYER_ACCELERATION;
                velocity.x = velocity.x.max(0.0);
            } else {
                acceleration.x = 0.0;
            }

            if key_down && !key_up {
                acceleration.y = -PLAYER_ACCELERATION;
                velocity.y = velocity.y.min(0.0);
            } else if key_up && !key_down {
                acceleration.y = PLAYER_ACCELERATION;
                velocity.y = velocity.y.max(0.0);
            } else {
                acceleration.y = 0.0;
            }

            acceleration.a = acceleration.normalize_or_zero() * PLAYER_ACCELERATION;
        }
    }
}

pub fn calculate_player_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Acceleration), With<Player>>,
) {
    for (mut velocity, acceleration) in &mut query {
        // Update the player's velocity based on the acceleration
        velocity.x += acceleration.x * time.delta_secs();
        velocity.y += acceleration.y * time.delta_secs();

        // Limit the player's speed to PLAYER_SPEED
        let speed = velocity.length();
        if speed > PLAYER_SPEED {
            velocity.a = velocity.normalize_or_zero() * PLAYER_SPEED;
        }
    }
}

// * Important: must be executed after methods that affect the velocity of objects
pub fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

pub fn spawn_bullets(
    time: Res<Time>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_query: Query<(&Transform, &Velocity, &mut BulletCooldown), With<Player>>,
) {
    for (player_transform, player_velocity, mut bullet_cooldown) in &mut player_query {
        bullet_cooldown.tick(time.delta());
        if keyboard_input.pressed(KeyCode::Space) {
            if bullet_cooldown.finished() {
                // Spawn a bullet at the paddle's position
                let bullet_position = player_transform;
                let bullet_velocity =
                    (player_velocity.normalize_or_zero() * BULLET_SPEED).extend(0.0);

                commands.spawn(BulletBundle::new(
                    &mut meshes,
                    &mut materials,
                    bullet_position,
                    &bullet_velocity,
                ));

                // Reset the bullet cooldown timer
                bullet_cooldown.reset();
            }
        }
    }
}

pub fn spawn_balls(
    time: Res<Time>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_query: Query<(&Transform, &Velocity, &mut BulletCooldown), With<Player>>,
) {
    for (player_transform, player_velocity, mut bullet_cooldown) in &mut player_query {
        bullet_cooldown.tick(time.delta());
        if keyboard_input.pressed(KeyCode::Space) {
            if bullet_cooldown.finished() {
                // Spawn a bullet at the paddle's position
                let bullet_position = player_transform;
                let bullet_velocity =
                    (player_velocity.normalize_or_zero() * BULLET_SPEED).extend(0.0);

                commands.spawn(BulletBundle::new(
                    &mut meshes,
                    &mut materials,
                    bullet_position,
                    &bullet_velocity,
                ));

                // Reset the bullet cooldown timer
                bullet_cooldown.reset();
            }
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
                collider_velocity.x = 0.0;
                collider_velocity.y = 0.0;
            }
        }
    }
}

pub fn check_for_ball_collisions(
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<&Transform, With<Collider>>,
    mut ball_collision_events: EventWriter<BallCollisionEvent>,
) {
    for (mut ball_velocity, ball_transform) in &mut ball_query {
        for collider_transform in &collider_query {
            let collision = collision(
                BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
                ball_transform.translation.truncate(),
                &Aabb2d::new(
                    collider_transform.translation.truncate(),
                    collider_transform.scale.truncate() / 2.,
                ),
            );

            if let Some(collision) = collision {
                // Writes a collision event so that other systems can react to the collision
                ball_collision_events.write_default();

                // Reflect the ball's velocity when it collides
                let mut reflect_x = false;
                let mut reflect_y = false;

                // Reflect only if the velocity is in the opposite direction of the collision
                // This prevents the ball from getting stuck inside the bar
                match collision {
                    Collision::Left => reflect_x = ball_velocity.x > 0.0,
                    Collision::Right => reflect_x = ball_velocity.x < 0.0,
                    Collision::Top => reflect_y = ball_velocity.y < 0.0,
                    Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                }

                // Reflect velocity on the x-axis if we hit something on the x-axis
                if reflect_x {
                    ball_velocity.x = -ball_velocity.x;
                }

                // Reflect velocity on the y-axis if we hit something on the y-axis
                if reflect_y {
                    ball_velocity.y = -ball_velocity.y;
                }
            }
        }
    }
}

// * It must execute before normal collision detection, so that the bullets can be despawned
pub fn check_for_bullet_collisions(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    collider_query: Query<&Transform, (With<Collider>, Without<Player>)>,
    mut bullet_collision_events: EventWriter<BallCollisionEvent>,
) {
    for (bullet_entity, bullet_transform) in bullet_query {
        for collider_transform in &collider_query {
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
