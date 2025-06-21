use bevy::{
    math::primitives::{Annulus, Circle, Rectangle, Triangle2d},
    prelude::Component,
};

#[derive(Component)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
    Triangle(Triangle2d),
    Annulus(Annulus),
}
