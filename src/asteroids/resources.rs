use bevy::{
    asset::Handle,
    audio::AudioSource,
    prelude::{Deref, Resource},
};

#[derive(Resource, Deref)]
pub struct CollisionSound {
    pub a: Handle<AudioSource>,
}
