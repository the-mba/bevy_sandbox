use super::resources::MouseWorldCoords;

use super::components::player::Player;

use avian2d::{math::*, prelude::*};
use bevy::prelude::*;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerAction>().add_systems(
            Update,
            (
                keyboard_input,
                mouse_input,
                gamepad_input,
                movement,
                rotation,
            )
                .chain(),
        );
    }
}

/// An event sent for a player input action.
#[derive(Event)]
pub enum PlayerAction {
    Move(Vector),
    Look(Vector),
}

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    movement: MovementBundle,
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    linear_damping: LinearDamping,
}

impl MovementBundle {
    pub const fn new(acceleration: Scalar) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            linear_damping: LinearDamping(0.95),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0)
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider) -> Self {
        // Create shape caster as a slightly smaller version of collider
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(caster_shape, Vector::ZERO, 0.0, Dir2::NEG_Y)
                .with_max_distance(10.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(mut self, acceleration: Scalar) -> Self {
        self.movement = MovementBundle::new(acceleration);
        self
    }
}

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<PlayerAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);

    let horizontal = right as i8 - left as i8;
    let vertical = up as i8 - down as i8;

    let direction = Vector::new(horizontal as Scalar, vertical as Scalar).normalize_or_zero();

    if direction.length() != 0.0 {
        movement_event_writer.write(PlayerAction::Move(direction));
    }

    // Leave this comment for reference on how to make it boost
    /* if keyboard_input.just_pressed(KeyCode::Space) {
        movement_event_writer.write(MovementAction::Jump);
    } */
}

/// Sends [`MovementAction`] events based on keyboard input.
fn mouse_input(
    mut movement_event_writer: EventWriter<PlayerAction>,
    mouse_input: Res<MouseWorldCoords>,
    player_transform: Single<&Transform, With<Player>>,
) {
    if let Some(mouse_position) = mouse_input.a {
        // Convert mouse position to a direction vector
        let direction =
            (mouse_position - player_transform.translation.truncate()).normalize_or_zero();
        movement_event_writer.write(PlayerAction::Look(direction));
    }
}

/// Sends [`MovementAction`] events based on gamepad input.
fn gamepad_input(mut movement_event_writer: EventWriter<PlayerAction>, gamepads: Query<&Gamepad>) {
    for gamepad in gamepads.iter() {
        movement_event_writer.write(PlayerAction::Move(gamepad.left_stick()));

        // Leave this comment for reference on how to make it boost
        /* if gamepad.just_pressed(GamepadButton::South) {
            movement_event_writer.write(MovementAction::Jump);
        } */
    }
}

/// Responds to [`MovementAction`] events and moves character controllers accordingly.
fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<PlayerAction>,
    mut controllers: Query<(&MovementAcceleration, &mut LinearVelocity, &mut Transform)>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs_f64().adjust_precision();

    for event in movement_event_reader.read() {
        for (movement_acceleration, mut linear_velocity, mut transform) in &mut controllers {
            match event {
                PlayerAction::Move(direction) => {
                    **linear_velocity += *direction * movement_acceleration.0 * delta_time;
                }
                PlayerAction::Look(direction) => {
                    transform.rotation = Quat::from_rotation_z(direction.y.atan2(direction.x))
                }
            }
        }
    }
}

fn rotation(q_gamepad: Query<&Gamepad>, mut q_player: Query<&mut Transform, With<Player>>) {
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
