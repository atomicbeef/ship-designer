pub mod fixed_update;
pub mod channels;
pub mod entity_lookup;
pub mod network_id;
pub mod player;
pub mod player_connection;
pub mod predefined_parts;
pub mod part;
pub mod compact_transform;
pub mod ship;
pub mod missile;

pub const PHYSICS_TIMESTEP: f32 = 1.0 / 60.0;