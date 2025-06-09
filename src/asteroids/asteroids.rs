//! A simplified implementation of the classic game "Breakout".
//!
//! Demonstrates Bevy's stepping capabilities if compiled with the `bevy_debug_stepping` feature.

use bevy::{prelude::*, window::WindowMode};

mod bundles;
mod components;
mod constants;
mod events;
mod resources;
mod systems;

pub use bundles::*;
pub use components::*;
pub use constants::*;
pub use events::*;
pub use resources::*;
pub use systems::*;

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
        // The stepping plugin is optional and can be used to control the game's update rate
        // .add_plugins(
        //     stepping::SteppingPlugin::default()
        //         .add_schedule(Update)
        //         .add_schedule(FixedUpdate)
        //         .at(Val::Percent(35.0), Val::Percent(50.0)),
        // )
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<BallCollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                spawn_bullets,
                spawn_balls,
                calculate_player_acceleration,
                calculate_player_velocity.after(calculate_player_acceleration),
                apply_velocity.after(calculate_player_velocity),
                window_collision.after(apply_velocity),
                check_for_ball_collisions,
                check_for_bullet_collisions,
                play_collision_sound,
                update_scoreboard,
            ),
        )
        .run();
}
