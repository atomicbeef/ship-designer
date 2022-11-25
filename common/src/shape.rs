use bevy::{prelude::{Component, Resource}, utils::HashMap};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::materials::Material;

#[derive(Clone)]
pub struct Shape {
    width: u8,
    height: u8,
    depth: u8,
    voxels: Vec<Material>,
}

impl Shape {
    pub fn new (width: u8, height: u8, depth: u8, voxels: Vec<Material>) -> Self {
        Self { width, height, depth, voxels }
    }

    fn pos_to_index(&self, x: usize, y: usize, z: usize) -> usize {
        self.width as usize * self.height as usize * z + y * self.width as usize + x
    }

    pub fn get(&self, x: u8, y: u8, z: u8) -> Material {
        let i = self.pos_to_index(x.into(), y.into(), z.into());
        self.voxels[i]
    }

    pub fn set(&mut self, x: u8, y: u8, z: u8, material: Material) {
        let i = self.pos_to_index(x.into(), y.into(), z.into());
        self.voxels[i] = material;
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShapeHandleId {
    id: u32
}

impl ShapeHandleId {
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl From<u32> for ShapeHandleId {
    fn from(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ShapeHandleType {
    ReadOnly,
    Unique
}

#[derive(Clone, Copy, Component)]
pub struct ShapeHandle {
    id: ShapeHandleId,
    handle_type: ShapeHandleType,
}

impl ShapeHandle {
    pub fn new(id: ShapeHandleId, handle_type: ShapeHandleType) -> Self {
        Self { id, handle_type }
    }

    pub fn id(&self) -> ShapeHandleId {
        self.id
    }

    pub fn handle_type(&self) -> ShapeHandleType {
        self.handle_type
    }
}

#[derive(Resource)]
pub struct Shapes {
    shapes: HashMap<ShapeHandleId, Shape>,
    current_shape_id: u32
}

impl Shapes {
    pub fn new() -> Self {
        Self { shapes: HashMap::new(), current_shape_id: 0 }
    }

    pub fn add(&mut self, shape: Shape) -> ShapeHandle {
        let id = ShapeHandleId::from(self.current_shape_id);
        self.shapes.insert(id, shape);
        self.current_shape_id += 1;

        ShapeHandle { id, handle_type: ShapeHandleType::ReadOnly }
    }

    pub fn get(&self, shape_handle: &ShapeHandle) -> Option<&Shape> {
        self.shapes.get(&shape_handle.id)
    }

    pub fn get_mut(&mut self, shape_handle: &mut ShapeHandle) -> Option<&mut Shape> {
        match shape_handle.handle_type {
            // If the handle is read-only, it must first be upgraded to be unique
            ShapeHandleType::ReadOnly => {
                let unique_shape = match self.shapes.get(&shape_handle.id) {
                    Some(shape) => shape.clone(),
                    None => { return None; }
                };

                let id = ShapeHandleId::from(self.current_shape_id);
                self.shapes.insert(id, unique_shape);
                self.current_shape_id += 1;

                shape_handle.id = id;
                shape_handle.handle_type = ShapeHandleType::Unique;

                self.shapes.get_mut(&id)
            },
            ShapeHandleType::Unique => self.shapes.get_mut(&shape_handle.id)
        }
    }

    /* TODO: Shape freeing */
}