use std::sync::Mutex;

use bevy::log::debug;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::utils::{hashbrown::hash_map, HashMap};
use crossbeam_channel::{Sender, Receiver};

use crate::materials::Material;
use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError};

// Voxels are 10^3 cm^3
pub const VOXEL_SIZE: f32 = 0.1;

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
        assert!(x < self.width as usize);
        assert!(y < self.height as usize);
        assert!(z < self.depth as usize);

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

    pub fn set_voxels(&mut self, new_voxels: &[Material]) {
        self.voxels.clear();
        self.voxels.extend_from_slice(new_voxels);
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

    pub fn center(&self) -> Vec3 {
        Vec3::new(
            self.width as f32 / 2.0 * VOXEL_SIZE,
            self.height as f32 / 2.0 * VOXEL_SIZE,
            self.depth as f32 / 2.0 * VOXEL_SIZE
        )
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

struct HandleDropped(ShapeId);

#[derive(Component, Reflect)]
pub struct ShapeHandle {
    id: ShapeId,
    #[reflect(ignore)]
    channel: Sender<HandleDropped>
}

impl ShapeHandle {
    pub fn id(&self) -> ShapeId {
        self.id
    }
}

impl Drop for ShapeHandle {
    fn drop(&mut self) {
        self.channel.send(HandleDropped(self.id)).unwrap();
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

pub struct FreedShapes(pub Box<[ShapeId]>);

#[derive(Resource)]
pub struct Shapes {
    shapes: HashMap<ShapeId, Shape>,
    current_shape_id: u32,
    ref_counts: Mutex<HashMap<ShapeId, usize>>,
    handle_dropped_channels: (Sender<HandleDropped>, Receiver<HandleDropped>),
}

impl Shapes {
    pub fn new() -> Self {
        Self {
            shapes: HashMap::new(),
            current_shape_id: 0,
            ref_counts: Mutex::new(HashMap::new()),
            handle_dropped_channels: (crossbeam_channel::unbounded()),
        }
    }

    pub fn add(&mut self, shape: Shape) -> ShapeHandle {
        let id = self.insert(shape);
        self.ref_counts.lock().unwrap().insert(id, 1);
        ShapeHandle { id, channel: self.handle_dropped_channels.0.clone() }
    }

    pub fn add_static(&mut self, shape: Shape) -> ShapeHandle {
        let id = self.insert(shape);
        self.ref_counts.lock().unwrap().insert(id, 2);
        ShapeHandle { id, channel: self.handle_dropped_channels.0.clone() }
    }

    pub fn get_handle(&self, shape_id: ShapeId) -> ShapeHandle {
        *self.ref_counts.lock().unwrap().entry(shape_id).or_insert(0) += 1;
        ShapeHandle { id: shape_id, channel: self.handle_dropped_channels.0.clone() }
    }

    pub fn get_shape_from_id(&self, shape_id: ShapeId) -> Option<&Shape> {
        self.shapes.get(&shape_id)
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
        self.ref_counts.lock().unwrap().insert(id, 1);

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

    fn get_unused_shape_ids(&mut self) -> Box<[ShapeId]> {
        let mut ref_counts = self.ref_counts.lock().unwrap();
        let mut unused_shape_ids = Vec::new();

        for handle_dropped in self.handle_dropped_channels.1.try_iter() {
            let count = match ref_counts.entry(handle_dropped.0) {
                hash_map::Entry::Occupied(mut entry) => {
                    let ref_count = entry.get_mut();
                    *ref_count -= 1;
                    *ref_count
                },
                hash_map::Entry::Vacant(_) => 0
            };
            
            if count == 0 {
                unused_shape_ids.push(handle_dropped.0);
                ref_counts.remove(&handle_dropped.0);
            }
        }

        unused_shape_ids.into_boxed_slice()
    }

}

pub fn free_shapes(mut shapes: ResMut<Shapes>, mut freed_shapes_writer: EventWriter<FreedShapes>) {
    let unused_shape_ids = shapes.get_unused_shape_ids();

    for id in unused_shape_ids.iter() {
        shapes.shapes.remove(id);
        debug!("Removed shape with ID {:?}", id);
    }

    freed_shapes_writer.send(FreedShapes(unused_shape_ids));
}