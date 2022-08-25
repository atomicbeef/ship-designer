use std::collections::HashMap;

use bevy::prelude::{Transform, Vec3, Entity, Component};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
    pub z: i32
}

impl From<Transform> for GridPos {
    fn from(transform: Transform) -> Self {
        GridPos {
            x: transform.translation.x as i32,
            y: transform.translation.y as i32,
            z: transform.translation.z as i32
        }
    }
}

impl From<&Transform> for GridPos {
    fn from(transform: &Transform) -> Self {
        GridPos {
            x: transform.translation.x as i32,
            y: transform.translation.y as i32,
            z: transform.translation.z as i32
        }
    }
}

impl From<Vec3> for GridPos {
    fn from(vec3: Vec3) -> Self {
        GridPos {
            x: vec3.x as i32,
            y: vec3.y as i32,
            z: vec3.z as i32
        }
    }
}

impl GridPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
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
}