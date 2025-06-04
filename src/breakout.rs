//! A simplified implementation of the classic game "Breakout".
//!
//! Demonstrates Bevy's stepping capabilities if compiled with the `bevy_debug_stepping` feature.

use bevy::{prelude::*, window::WindowMode};
use constants::*;
use events::*;
use resources::*;
use systems::*;

pub mod constants {
    use bevy::prelude::*;

    pub const SPAWN_BALLS: bool = false;

    // These constants are defined in `Transform` units.
    // Using the default 2D camera they correspond 1:1 with screen pixels.
    pub const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
    pub const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
    pub const PADDLE_SPEED: f32 = 500.0;
    // How close can the paddle get to the wall
    pub const PADDLE_PADDING: f32 = 10.0;

    // We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
    pub const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
    pub const BALL_DIAMETER: f32 = 30.;
    pub const BALL_SPEED: f32 = 400.0;
    pub const BALL_SPEED_MULTIPLIER: f32 = 1.05; // Increase ball speed by 5% on each brick hit
    pub const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

    pub const WALL_THICKNESS: f32 = 10.0;
    // x coordinates
    pub const LEFT_WALL: f32 = -800.;
    pub const RIGHT_WALL: f32 = 800.;
    // y coordinates
    pub const BOTTOM_WALL: f32 = -450.;
    pub const TOP_WALL: f32 = 400.;

    pub const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
    // Normal, Speed, ExtraBall
    pub const BRICK_TYPE_NORMAL_WEIGHT: f32 = 0.7;
    pub const BRICK_TYPE_SPEED_WEIGHT: f32 = 0.2;
    pub const BRICK_TYPE_EXTRA_BALL_WEIGHT: f32 = 0.1;

    // These values are exact
    pub const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
    pub const GAP_BETWEEN_BRICKS: f32 = 5.0;
    // These values are lower bounds, as the number of bricks is computed
    pub const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
    pub const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

    pub const SCOREBOARD_FONT_SIZE: f32 = 33.0;
    pub const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

    pub const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
    pub const PADDLE_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);
    pub const BALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
    pub const BRICK_NORMAL_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
    pub const BRICK_SPEED_COLOR: Color = Color::srgb(0.5, 1.0, 0.5);
    pub const BRICK_EXTRA_BALL_COLOR: Color = Color::srgb(1.0, 1.0, 0.5);
    pub const WALL_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
    pub const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
    pub const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
}

pub mod components {
    use super::constants::*;
    use bevy::prelude::*;
    #[derive(Component)]
    pub struct Paddle;

    #[derive(Component)]
    pub struct Ball;

    #[derive(Component, Deref, DerefMut)]
    pub struct Velocity(pub Vec2);

    pub enum BrickType {
        Normal,
        Speed,
        ExtraBall,
    }

    impl BrickType {
        pub fn color(&self) -> Color {
            match self {
                BrickType::Normal => BRICK_NORMAL_COLOR,
                BrickType::Speed => BRICK_SPEED_COLOR,
                BrickType::ExtraBall => BRICK_EXTRA_BALL_COLOR,
            }
        }

        pub fn weights() -> Vec<f32> {
            Vec::from([
                BRICK_TYPE_NORMAL_WEIGHT,
                BRICK_TYPE_SPEED_WEIGHT,
                BRICK_TYPE_EXTRA_BALL_WEIGHT,
            ])
        }

        pub fn random() -> Self {
            let random = getrandom::u32().unwrap_or(0);
            let weights = Self::weights();
            let total_weight: f32 = weights.iter().sum();
            let mut cumulative_weight = 0.0;

            for (i, &weight) in weights.iter().enumerate() {
                cumulative_weight += weight / total_weight;
                if (random as f32) / (u32::MAX as f32) < cumulative_weight {
                    return match i {
                        0 => BrickType::Normal,
                        1 => BrickType::Speed,
                        _ => BrickType::ExtraBall,
                    };
                }
            }

            BrickType::Normal // Fallback, should not happen
        }
    }

    impl Default for BrickType {
        fn default() -> Self {
            BrickType::Normal
        }
    }

    #[derive(Component)]
    pub struct Brick {
        pub r#type: BrickType,
    }

    // Default must be implemented to define this as a required component for the Wall component below
    #[derive(Component, Default)]
    pub struct Collider;

    // This is a collection of the components that define a "Wall" in our game
    #[derive(Component)]
    #[require(Sprite, Transform, Collider)]
    pub struct Wall;

    /// Which side of the arena is this wall located on?
    pub enum WallLocation {
        Left,
        Right,
        Bottom,
        Top,
    }

    impl WallLocation {
        /// Location of the *center* of the wall, used in `transform.translation()`
        fn position(&self) -> Vec2 {
            match self {
                WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
                WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
                WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
                WallLocation::Top => Vec2::new(0., TOP_WALL),
            }
        }

        /// (x, y) dimensions of the wall, used in `transform.scale()`
        fn size(&self) -> Vec2 {
            let arena_height = TOP_WALL - BOTTOM_WALL;
            let arena_width = RIGHT_WALL - LEFT_WALL;
            // Make sure we haven't messed up our constants
            assert!(arena_height > 0.0);
            assert!(arena_width > 0.0);

            match self {
                WallLocation::Left | WallLocation::Right => {
                    Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
                }
                WallLocation::Bottom | WallLocation::Top => {
                    Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
                }
            }
        }
    }

    impl Wall {
        // This "builder method" allows us to reuse logic across our wall entities,
        // making our code easier to read and less prone to bugs when we change the logic
        // Notice the use of Sprite and Transform alongside Wall, overwriting the default values defined for the required components
        pub fn new(location: WallLocation) -> (Wall, Sprite, Transform) {
            (
                Wall,
                Sprite::from_color(WALL_COLOR, Vec2::ONE),
                Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
            )
        }
    }

    #[derive(Component)]
    pub struct ScoreboardUi;
}

pub mod bundles {
    use super::components::*;
    use super::constants::*;
    use crate::resources::*;
    use bevy::prelude::*;
    #[derive(Bundle)]
    pub struct PaddleBundle {
        pub sprite: Sprite,
        pub transform: Transform,
        pub paddle: Paddle,
        pub collider: Collider,
    }

    impl Default for PaddleBundle {
        fn default() -> Self {
            Self {
                sprite: Sprite {
                    color: PADDLE_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    scale: PADDLE_SIZE.extend(1.0),
                    ..default()
                },
                paddle: Paddle,
                collider: Collider,
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
            ball_speed: &ResMut<Speed>,
        ) -> Self {
            Self {
                mesh: Mesh2d(meshes.add(Circle::default())),
                material: MeshMaterial2d(materials.add(BALL_COLOR)),
                transform: Transform::from_translation(BALL_STARTING_POSITION)
                    .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.0)),
                ball: Ball,
                velocity: Velocity(INITIAL_BALL_DIRECTION.normalize() * ball_speed.0),
            }
        }
    }

    #[derive(Bundle)]
    pub struct BrickBundle {
        pub sprite: Sprite,
        pub transform: Transform,
        pub brick: Brick,
        pub collider: Collider,
    }

    impl BrickBundle {
        pub fn new(brick_position: Vec2, r#type: BrickType) -> Self {
            Self {
                sprite: Sprite {
                    color: BrickType::color(&r#type),
                    ..default()
                },
                transform: Transform {
                    translation: brick_position.extend(0.0),
                    scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
                    ..default()
                },
                brick: Brick { r#type: r#type },
                collider: Collider,
            }
        }
    }
}

pub mod resources {
    use super::constants::*;
    use bevy::prelude::*;
    #[derive(Resource, Deref)]
    pub struct CollisionSound(pub Handle<AudioSource>);

    // This resource tracks the game's score
    #[derive(Resource, Deref, DerefMut)]
    pub struct Score(pub usize);

    // This resource tracks the balls' speed
    #[derive(Resource, Deref, DerefMut)]
    pub struct Speed(pub f32);
    impl Default for Speed {
        fn default() -> Self {
            Speed(BALL_SPEED)
        }
    }
}

pub mod events {
    use bevy::prelude::*;
    #[derive(Event, Default)]
    pub struct CollisionEvent;
}

pub mod systems {
    use super::bundles::*;
    use super::components::*;
    use super::constants::*;
    use super::events::*;
    use super::resources::*;
    use bevy::math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume};
    use bevy::prelude::*;

    // Add the game's entities to our world
    pub fn setup(
        mut windows: Query<&mut Window>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        asset_server: Res<AssetServer>,
        ball_speed: ResMut<Speed>,
    ) {
        // Set up the window
        let mut window = windows.single_mut().expect("No window found");
        window.set_maximized(true);

        // Camera
        commands.spawn(Camera2d);

        // Sound
        let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
        commands.insert_resource(CollisionSound(ball_collision_sound));

        // Paddle
        let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

        let mut paddle_bundle = PaddleBundle::default();
        paddle_bundle.transform.translation.y = paddle_y;
        commands.spawn(paddle_bundle);

        // Ball
        commands.spawn(BallBundle::new(&mut meshes, &mut materials, &ball_speed));

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
                TextSpan::default(),
                TextFont {
                    font_size: SCOREBOARD_FONT_SIZE,
                    ..default()
                },
                TextColor(SCORE_COLOR),
            )],
        ));

        // Walls
        commands.spawn(Wall::new(WallLocation::Left));
        commands.spawn(Wall::new(WallLocation::Right));
        commands.spawn(Wall::new(WallLocation::Bottom));
        commands.spawn(Wall::new(WallLocation::Top));

        // Bricks
        let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
        let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
        let total_height_of_bricks =
            TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

        assert!(total_width_of_bricks > 0.0);
        assert!(total_height_of_bricks > 0.0);

        // Given the space available, compute how many rows and columns of bricks we can fit
        let n_columns =
            (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
        let n_rows =
            (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
        let n_vertical_gaps = n_columns - 1;

        // Because we need to round the number of columns,
        // the space on the top and sides of the bricks only captures a lower bound, not an exact value
        let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0;
        let left_edge_of_bricks = center_of_bricks
        // Space taken up by the bricks
        - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
        // Space taken up by the gaps
        - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

        // In Bevy, the `translation` of an entity describes the center point,
        // not its bottom-left corner
        let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.;
        let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.;

        // let mut rng_color = rand::thread_rng();
        for row in 0..n_rows {
            for column in 0..n_columns {
                let brick_position = Vec2::new(
                    offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                    offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
                );

                // brick
                commands.spawn(BrickBundle::new(brick_position, BrickType::random()));
            }
        }
    }

    pub fn move_paddle(
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut paddle_transform: Single<&mut Transform, With<Paddle>>,
        time: Res<Time>,
    ) {
        let mut direction = 0.0;

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction += 1.0;
        }

        // Calculate the new horizontal paddle position based on player input
        let new_paddle_position =
            paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_secs();

        // Update the paddle position,
        // making sure it doesn't cause the paddle to leave the arena
        let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
        let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

        paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
    }

    pub fn apply_velocity(
        mut query: Query<(&mut Transform, &Velocity)>,
        ball_speed: Res<Speed>,
        time: Res<Time>,
    ) {
        for (mut transform, velocity) in &mut query {
            let velocity = velocity.normalize_or(Vec2::ONE) * ball_speed.0;
            transform.translation.x += velocity.x * time.delta_secs();
            transform.translation.y += velocity.y * time.delta_secs();
        }
    }

    pub fn update_scoreboard(
        score: Res<Score>,
        score_root: Single<Entity, (With<ScoreboardUi>, With<Text>)>,
        mut writer: TextUiWriter,
    ) {
        *writer.text(*score_root, 1) = score.to_string();
    }

    pub fn check_for_collisions(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        mut score: ResMut<Score>,
        mut ball_speed: ResMut<Speed>,
        mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
        collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
        mut collision_events: EventWriter<CollisionEvent>,
    ) {
        for (mut ball_velocity, ball_transform) in &mut ball_query {
            for (collider_entity, collider_transform, maybe_brick) in &collider_query {
                let collision = ball_collision(
                    BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
                    Aabb2d::new(
                        collider_transform.translation.truncate(),
                        collider_transform.scale.truncate() / 2.,
                    ),
                );

                if let Some(collision) = collision {
                    // Writes a collision event so that other systems can react to the collision
                    collision_events.write_default();

                    if let Some(brick) = maybe_brick {
                        // Bricks should be despawned and increment the scoreboard on collision
                        commands.entity(collider_entity).despawn();
                        **score += 1;

                        // If the brick was of type Speed, increase the ball speed
                        match brick.r#type {
                            BrickType::Normal => {}
                            BrickType::Speed => {
                                ball_speed.0 *= BALL_SPEED_MULTIPLIER;
                            }
                            BrickType::ExtraBall => {
                                // If the brick was of type ExtraBall, spawn a new ball
                                commands.spawn(BallBundle::new(
                                    &mut meshes,
                                    &mut materials,
                                    &ball_speed,
                                ));
                            }
                        }
                    }

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

    pub fn play_collision_sound(
        mut commands: Commands,
        mut collision_events: EventReader<CollisionEvent>,
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

    // Returns `Some` if `ball` collides with `bounding_box`.
    // The returned `Collision` is the side of `bounding_box` that `ball` hit.
    fn ball_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
        if !ball.intersects(&bounding_box) {
            return None;
        }

        let closest = bounding_box.closest_point(ball.center());
        let offset = ball.center() - closest;
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
}

fn main() {
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            mode: WindowMode::Windowed,
            title: "Breakout de Mario".to_string(),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    };
    App::new()
        .add_plugins(DefaultPlugins.set(window_plugin))
        // .add_plugins(
        //     stepping::SteppingPlugin::default()
        //         .add_schedule(Update)
        //         .add_schedule(FixedUpdate)
        //         .at(Val::Percent(35.0), Val::Percent(50.0)),
        // )
        .insert_resource(Score(0))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Speed::default())
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        // Add our gameplay simulation systems to the fixed timestep schedule
        // which runs at 64 Hz by default
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                move_paddle,
                check_for_collisions,
                play_collision_sound,
            )
                // `chain`ing systems together runs them in order
                .chain(),
        )
        .add_systems(Update, update_scoreboard)
        .run();
}
