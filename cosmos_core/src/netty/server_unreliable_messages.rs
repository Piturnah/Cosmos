use bevy::prelude::{Component, Entity};
use serde::{Deserialize, Serialize};

use crate::structure::ship::ship_movement::ShipMovement;

use super::netty_rigidbody::NettyRigidBody;

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerUnreliableMessages {
    BulkBodies {
        bodies: Vec<(Entity, NettyRigidBody)>,
        time_stamp: u32,
    },
    SetMovement {
        movement: ShipMovement,
        ship_entity: Entity,
    },
}
