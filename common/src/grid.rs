use bevy::utils::hashbrown::HashMap;
use bevy::prelude::{Transform, Vec3, Entity, Component};

use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GridPos {
    pub x: i16,
    pub y: i16,
    pub z: i16
}

impl From<Transform> for GridPos {
    fn from(transform: Transform) -> Self {
        GridPos {
            x: transform.translation.x as i16,
            y: transform.translation.y as i16,
            z: transform.translation.z as i16
        }
    }
}

impl From<&Transform> for GridPos {
    fn from(transform: &Transform) -> Self {
        GridPos {
            x: transform.translation.x as i16,
            y: transform.translation.y as i16,
            z: transform.translation.z as i16
        }
    }
}

impl From<GridPos> for Transform {
    fn from(pos: GridPos) -> Self {
        Transform {
            translation: Vec3::new(
                pos.x as f32,
                pos.y as f32,
                pos.z as f32
            ),
            ..Default::default()
        }
    }
}

impl From<&GridPos> for Transform {
    fn from(pos: &GridPos) -> Self {
        Transform {
            translation: Vec3::new(
                pos.x as f32,
                pos.y as f32,
                pos.z as f32
            ),
            ..Default::default()
        }
    }
}

impl From<Vec3> for GridPos {
    fn from(vec3: Vec3) -> Self {
        GridPos {
            x: vec3.x as i16,
            y: vec3.y as i16,
            z: vec3.z as i16
        }
    }
}

impl GridPos {
    pub fn new(x: i16, y: i16, z: i16) -> Self {
        GridPos { x, y, z }
    }
}

#[derive(Component)]
pub struct Grid {
    grid: HashMap<GridPos, Entity>,
}

impl Grid {
    pub fn new() -> Self {
        Grid {
            grid: HashMap::new()
        }
    }

    pub fn get(&self, pos: &GridPos) -> Option<Entity> {
        match self.grid.get(pos) {
            Some(entity) => Some(*entity),
            None => None
        }
    }

    pub fn set(&mut self, pos: &GridPos, entity: Option<Entity>) {
        match entity {
            Some(x) => { self.grid.insert(*pos, x); },
            None => { self.grid.remove(pos); },
        }
    }

    pub fn exists_at(&self, pos: &GridPos) -> bool {
        self.grid.contains_key(pos)
    }

    pub fn positions(&self) -> Vec<GridPos> {
        self.grid.keys().cloned().collect()
    }
}

impl PacketSerialize<GridPos> for Packet {
    fn write(&mut self, pos: GridPos) {
        self.write(pos.x);
        self.write(pos.y);
        self.write(pos.z);
    }
}

impl PacketDeserialize<GridPos> for Packet {
    fn read(&mut self) -> Result<GridPos, PacketError> {
        let x: i16 = self.read()?;
        let y: i16 = self.read()?;
        let z: i16 = self.read()?;
    
        Ok(GridPos::new(x, y, z))
    }
}