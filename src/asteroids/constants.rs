use bevy::{
    color::Color,
    input::{gamepad::GamepadButton, keyboard::KeyCode},
    prelude::Vec2,
    ui::Val,
};

// * PLAYER *
// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
pub const PLAYER_SIZE: Vec2 = Vec2::new(20.0, 20.0);
pub const PLAYER_STARTING_POSITION: Vec2 = Vec2::new(0.0, 0.0);
pub const PLAYER_STARTING_ORIENTATION: Vec2 = Vec2::new(0.0, 1.0);
pub const PLAYER_STARTING_VELOCITY: Vec2 = Vec2::new(0.0, 0.0);
pub const PLAYER_SPEED: f32 = 500.0;
pub const PLAYER_STARTING_ACCELERATION: Vec2 = Vec2::new(0.0, 0.0);
pub const PLAYER_ACCELERATION: f32 = 50000.0;
pub const PLAYER_BRAKING_MULTIPLIER: f32 = 2.0;

// * BALL *
pub const BALL_DIAMETER: f32 = 30.0;
pub const BALL_SPEED: f32 = 400.0;
pub const BALL_COOLDOWN: f32 = 1.0; // Time in seconds before the next ball spawns

// * BULLET *
pub const BULLET_RADIUS: f32 = 5.0;
pub const BULLET_SPEED: f32 = 1500.0;
pub const BULLET_COOLDOWN: f32 = 0.05; // Time in seconds before the next bullet can be fired

// * LASER *
pub const LASER_LENGTH: f32 = 100.0;
pub const LASER_WIDTH: f32 = 5.0;
pub const LASER_COOLDOWN: f32 = 0.1; // Time in seconds before the next bullet can be fired
pub const LASER_LIFE: f32 = 99999.0; // Time in seconds before the laser despawns

// * SCOREBOARD *
pub const SCOREBOARD_FONT_SIZE: f32 = 33.0;
pub const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

// * COLORS *
pub const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const PLAYER_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);
pub const BALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
pub const BULLET_COLOR: Color = Color::srgb(0.0, 0.0, 1.0);
pub const LASER_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
pub const BRICK_NORMAL_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
pub const BRICK_SPEED_COLOR: Color = Color::srgb(0.5, 1.0, 0.5);
pub const BRICK_EXTRA_BALL_COLOR: Color = Color::srgb(1.0, 1.0, 0.5);
pub const WALL_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
pub const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
pub const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);

// * KEYBOARD *
pub const KEYBOARD_BULLET_BUTTON: KeyCode = KeyCode::Space;
pub const KEYBOARD_LASER_BUTTON: KeyCode = KeyCode::ShiftLeft;

// * GAMEPAD *
pub const GAMEPAD_BULLET_BUTTON: GamepadButton = GamepadButton::RightTrigger;
pub const GAMEPAD_LASER_BUTTON: GamepadButton = GamepadButton::RightTrigger2;
