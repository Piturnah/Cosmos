use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use bevy_renet::renet::RenetServer;
use cosmos_core::{
    netty::{cosmos_encoder, netty_rigidbody::NettyRigidBody, NettyChannel},
    physics::location::Location,
    structure::{
        asteroid::{asteroid_netty::AsteroidServerMessages, Asteroid},
        Structure,
    },
};

use crate::netty::sync::entities::RequestedEntityEvent;

fn on_request_asteroid(
    mut event_reader: EventReader<RequestedEntityEvent>,
    query: Query<(&Structure, &Transform, &Location, &Velocity), With<Asteroid>>,
    mut server: ResMut<RenetServer>,
) {
    for ev in event_reader.iter() {
        if let Ok((structure, transform, location, velocity)) = query.get(ev.entity) {
            server.send_message(
                ev.client_id,
                NettyChannel::Asteroids.id(),
                cosmos_encoder::serialize(&AsteroidServerMessages::Asteroid {
                    body: NettyRigidBody::new(velocity, transform.rotation, *location),
                    entity: ev.entity,
                    width: structure.chunks_width() as u32,
                    height: structure.chunks_height() as u32,
                    length: structure.chunks_length() as u32,
                }),
            );
        }
    }
}

pub(super) fn register(app: &mut App) {
    app.add_system(on_request_asteroid);
}
