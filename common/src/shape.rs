use bevy::ecs::reflect::ReflectComponent;
use bevy::prelude::{Component, Resource};
use bevy::reflect::Reflect;
use bevy::utils::HashMap;

use crate::materials::Material;
use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct ShapeId {
    id: u32
}

impl ShapeId {
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl From<u32> for ShapeId {
    fn from(id: u32) -> Self {
        Self { id }
    }
}

impl PacketSerialize for ShapeId {
    fn serialize(&self, packet: &mut Packet) {
        self.id.serialize(packet);
    }
}

impl PacketDeserialize for ShapeId {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let id = u32::deserialize(packet)?;
        Ok(Self::from(id))
    }
}

#[derive(Clone)]
pub struct Shape {
    width: u8,
    height: u8,
    depth: u8,
    voxels: Vec<Material>,
    parent_shape_id: Option<ShapeId>
}

impl Shape {
    pub fn new (width: u8, height: u8, depth: u8, voxels: Vec<Material>, parent_shape_id: Option<ShapeId>) -> Self {
        Self { width, height, depth, voxels, parent_shape_id }
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

    pub fn voxels(&self) -> &[Material] {
        &self.voxels
    }

    pub fn parent_shape_id(&self) -> Option<ShapeId> {
        self.parent_shape_id
    }

    pub fn clone_as_child_of(&self, parent_shape_id: ShapeId) -> Self {
        Self {
            width: self.width,
            height: self.height,
            depth: self.depth,
            voxels: self.voxels.clone(),
            parent_shape_id: Some(parent_shape_id)
        }
    }
}

impl PacketSerialize for &Shape {
    fn serialize(&self, packet: &mut Packet) {
        self.width.serialize(packet);
        self.height.serialize(packet);
        self.depth.serialize(packet);
        self.voxels.as_slice().serialize(packet);
        self.parent_shape_id.serialize(packet);
    }
}

impl PacketDeserialize for Shape {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let width = u8::deserialize(packet)?;
        let height = u8::deserialize(packet)?;
        let depth = u8::deserialize(packet)?;
        let voxels: Vec<Material> = Vec::deserialize(packet)?;
        let parent_shape_id: Option<ShapeId> = Option::deserialize(packet)?;

        Ok(Self::new(width, height, depth, voxels, parent_shape_id))
    }
}

#[derive(Clone, Component, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct ShapeHandle {
    id: ShapeId
}

impl Default for ShapeHandle {
    fn default() -> Self {
        Self { id: ShapeId::from(0) }
    }
}

impl ShapeHandle {
    pub fn new(id: ShapeId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> ShapeId {
        self.id
    }
}

pub enum ShapeNetworkRepr {
    Predefined(ShapeId),
    Child(Shape)
}

impl PacketSerialize for ShapeNetworkRepr {
    fn serialize(&self, packet: &mut Packet) {
        match self {
            Self::Predefined(shape_id) => {
                false.serialize(packet);
                shape_id.serialize(packet);
            },
            Self::Child(shape) => {
                true.serialize(packet);
                shape.serialize(packet);
            }
        }
    }
}

impl PacketDeserialize for ShapeNetworkRepr {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let is_child = bool::deserialize(packet)?;

        if is_child {
            let shape = Shape::deserialize(packet)?;
            Ok(Self::Child(shape))
        } else {
            let shape_id = ShapeId::deserialize(packet)?;
            Ok(Self::Predefined(shape_id))
        }
    }
}

#[derive(Resource)]
pub struct Shapes {
    shapes: HashMap<ShapeId, Shape>,
    current_shape_id: u32
}

impl Shapes {
    pub fn new() -> Self {
        Self { shapes: HashMap::new(), current_shape_id: 0 }
    }

    pub fn add(&mut self, shape: Shape) -> ShapeHandle {
        let id = self.insert(shape);
        ShapeHandle { id }
    }

    pub fn get(&self, shape_handle: &ShapeHandle) -> Option<&Shape> {
        self.shapes.get(&shape_handle.id)
    }

    pub fn get_mut(&mut self, shape_handle: &mut ShapeHandle) -> Option<&mut Shape> {
        match self.is_child_shape(shape_handle) {
            Some(is_child) => {
                if is_child {
                    return self.shapes.get_mut(&shape_handle.id);
                }
            },
            None => { return None; }
        }

        let new_shape = self.clone_shape_from_shape_id(shape_handle.id);
        let id = self.insert(new_shape);

        shape_handle.id = id;

        self.shapes.get_mut(&id)
    }

    pub fn clone_shape_from_shape_id(&self, parent_shape_id: ShapeId) -> Shape {
        let parent_shape = self.shapes.get(&parent_shape_id).unwrap();
        parent_shape.clone_as_child_of(parent_shape_id)
    }

    fn insert(&mut self, new_shape: Shape) -> ShapeId {
        let id = ShapeId::from(self.current_shape_id);
        self.shapes.insert(id, new_shape);
        self.current_shape_id += 1;
        id
    }

    fn is_child_shape(&self, shape_handle: &ShapeHandle) -> Option<bool> {
        let shape = self.shapes.get(&shape_handle.id)?;
        Some(shape.parent_shape_id.is_some())
    }

    /* TODO: Shape freeing */
}