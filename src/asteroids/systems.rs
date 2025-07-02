use super::bundles::*;
use super::components::*;
use super::constants::*;
use super::events::*;
use super::resources::*;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::RectangleMeshBuilder;
// Commented in case I need to use it later
// use bevy_tnua_avian2d::*;

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

    // Spawn the ground.
    commands.spawn((
        Mesh2d(meshes.add(RectangleMeshBuilder::new(600.0, 50.0))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(0.0, -300.0, 0.0),
        RigidBody::Static,
        Collider::half_space(Vec2::Y),
    ));

    // Spawn a little platform for the player to jump on.
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(40.0, 10.0))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(-60.0, 2.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(4.0, 1.0),
    ));
}

pub fn detect_bullet_player_collision(
    mut commands: Commands,
    q_bullets: Query<(Entity, &CollidingEntities), With<Bullet>>,
    q_player: Single<Entity, With<Player>>,
) {
    for (entity, colliding_entities) in &q_bullets {
        // Check if the bullet collided with a player
        if let None = colliding_entities
            .iter()
            .find(|&&e| e == q_player.to_owned())
        {
            commands.entity(entity).remove::<JustFired>();
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
                let bullet_velocity =
                    (player_transform.rotation.mul_vec3(Vec3::X) * BULLET_SPEED).truncate();

                commands
                    .spawn(BulletBundle::new(
                        &mut r_mesh,
                        &mut r_material,
                        *bullet_position,
                        bullet_velocity,
                    ))
                    .insert(JustFired);

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
    mut destroyed_events: EventReader<DestroyedEvent>,
    score_query: Query<(Entity, &mut Score), (With<ScoreboardUi>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    if !destroyed_events.is_empty() {
        for (score_board_ui_entity, mut score) in score_query {
            // It is assumed the score is added to all players (via their Score component).
            score.a += destroyed_events.len();
            *writer.text(score_board_ui_entity, 1) = score.to_string();
        }
        destroyed_events.clear();
    }
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

pub fn update_mouse_coords(
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut mouse_coords: ResMut<MouseCoords>,
    mut mouse_world_coords: ResMut<MouseWorldCoords>,
) {
    mouse_coords.a = window.cursor_position();
    mouse_world_coords.a = mouse_coords.map(|pos| {
        let (camera, camera_transform) = camera.into_inner();
        camera
            .viewport_to_world_2d(camera_transform, pos)
            .unwrap_or(vec2(0.0, 0.0))
    });
}
