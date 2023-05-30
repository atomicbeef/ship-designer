use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::network_id::NetworkId;

#[derive(Component)]
pub struct Ship;

#[derive(Bundle)]
pub struct ShipBundle {
    pub transform: TransformBundle,
    pub visibility: VisibilityBundle,
    pub network_id: NetworkId,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub ship: Ship,
}

impl Default for ShipBundle {
    fn default() -> Self {
        Self {
            transform: TransformBundle::default(),
            visibility: VisibilityBundle::default(),
            network_id: NetworkId::from(0),
            rigid_body: RigidBody::Dynamic,
            velocity: Velocity::default(),
            ship: Ship,
        }
    }
}