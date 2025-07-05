//! A simplified implementation of the classic game "Breakout".
//!
//! Demonstrates Bevy's stepping capabilities if compiled with the `bevy_debug_stepping` feature.

use avian2d::prelude::*;
use bevy::{prelude::*, window::WindowMode};

mod bundles;
mod character_controller_plugin;
mod components;
mod constants;
mod events;
mod resources;
mod systems;

pub use bundles::*;
pub use character_controller_plugin::*;
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
        .add_plugins((
            DefaultPlugins.set(window_plugin),
            PhysicsPlugins::default().with_length_unit(20.0),
            CharacterControllerPlugin,
        ))
        // The stepping plugin is optional and can be used to control the game's update rate
        // .add_plugins(
        //     stepping::SteppingPlugin::default()
        //         .add_schedule(Update)
        //         .add_schedule(FixedUpdate)
        //         .at(Val::Percent(35.0), Val::Percent(50.0)),
        // )
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(BallCooldown::default())
        .insert_resource(Gravity(Vec2::ZERO))
        .insert_resource(MouseCoords::default())
        .insert_resource(MouseWorldCoords::default())
        .add_event::<BallCollisionEvent>()
        .add_event::<DestroyedEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_mouse_coords,
                spawn_attacks,
                spawn_balls,
                despawn,
                play_collision_sound,
                update_scoreboard,
            )
                .chain(),
        )
        .run();
}
