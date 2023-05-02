use std::sync::Mutex;

use bevy::ecs::system::Command;
use bevy::log::debug;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::utils::{hashbrown::hash_map, HashMap};
use crossbeam_channel::{Sender, Receiver};

use packets::{Packet, PacketSerialize, PacketDeserialize, PacketError};

use events::*;
use colliders::{RegenerateColliders, remove_unused_colliders};
use materials::Material;
use packets_derive::{PacketSerialize, PacketDeserialize};

use self::colliders::PartCollider;

pub mod colliders;
pub mod events;
pub mod materials;

// Voxels are 10^3 cm^3
pub const VOXEL_SIZE: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, PacketSerialize, PacketDeserialize)]
pub struct PartId {
    id: u32
}

impl PartId {
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl From<u32> for PartId {
    fn from(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoxelPos {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl VoxelPos {
    pub fn new(x: u8, y: u8, z: u8) -> Self {
        Self { x, y, z }
    }
}

impl From<Vec3> for VoxelPos {
    fn from(value: Vec3) -> Self {
        Self::new(value.x as u8, value.y as u8, value.z as u8)
    }
}

impl From<VoxelPos> for Vec3 {
    fn from(value: VoxelPos) -> Self {
        Self::new(value.x as f32, value.y as f32, value.z as f32)
    }
}

#[derive(Clone, PacketSerialize, PacketDeserialize)]
pub struct Part {
    width: u8,
    height: u8,
    depth: u8,
    voxels: Vec<Material>,
    parent_part_id: Option<PartId>
}

impl Part {
    pub fn new (width: u8, height: u8, depth: u8, voxels: Vec<Material>, parent_part_id: Option<PartId>) -> Self {
        Self { width, height, depth, voxels, parent_part_id }
    }

    pub fn voxel_to_index(&self, x: u8, y: u8, z: u8) -> usize {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        debug_assert!(z < self.depth);

        let width = self.width as usize;
        let height = self.height as usize;
        let x = x as usize;
        let y = y as usize;
        let z = z as usize;

        width * height * z + y * width + x
    }

    pub fn voxel_is_in_part(&self, pos: VoxelPos) -> bool {
        pos.x < self.width && pos.y < self.height && pos.z < self.depth
    }

    pub fn get(&self, pos: VoxelPos) -> Material {
        let i = self.voxel_to_index(pos.x, pos.y, pos.z);
        self.voxels[i]
    }

    pub fn get_index(&self, i: usize) -> Material {
        self.voxels[i]
    }

    pub fn set(&mut self, pos: VoxelPos, material: Material) {
        let i = self.voxel_to_index(pos.x, pos.y, pos.z);
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

    pub fn size(&self) -> u32 {
        self.width as u32 * self.height as u32 * self.depth as u32
    }

    pub fn voxels(&self) -> &[Material] {
        &self.voxels
    }

    pub fn set_voxels(&mut self, new_voxels: &[Material]) {
        self.voxels.clear();
        self.voxels.extend_from_slice(new_voxels);
    }

    pub fn parent_part_id(&self) -> Option<PartId> {
        self.parent_part_id
    }

    pub fn clone_as_child_of(&self, parent_part_id: PartId) -> Self {
        Self {
            width: self.width,
            height: self.height,
            depth: self.depth,
            voxels: self.voxels.clone(),
            parent_part_id: Some(parent_part_id)
        }
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(
            self.width as f32 / 2.0 * VOXEL_SIZE,
            self.height as f32 / 2.0 * VOXEL_SIZE,
            self.depth as f32 / 2.0 * VOXEL_SIZE
        )
    }

    pub fn is_empty(&self) -> bool {
        self.voxels.iter().all(|&material| material == Material::Empty)
    }
}

struct HandleDropped(PartId);

#[derive(Component, Reflect)]
pub struct PartHandle {
    id: PartId,
    #[reflect(ignore)]
    channel: Sender<HandleDropped>
}

impl PartHandle {
    pub fn id(&self) -> PartId {
        self.id
    }
}

impl Drop for PartHandle {
    fn drop(&mut self) {
        self.channel.send(HandleDropped(self.id)).unwrap();
    }
}

pub enum PartNetworkRepr {
    Predefined(PartId),
    Child(Part)
}

impl PacketSerialize for PartNetworkRepr {
    fn serialize(&self, packet: &mut Packet) {
        match self {
            Self::Predefined(part_id) => {
                false.serialize(packet);
                part_id.serialize(packet);
            },
            Self::Child(part) => {
                true.serialize(packet);
                part.serialize(packet);
            }
        }
    }
}

impl PacketDeserialize for PartNetworkRepr {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let is_child = bool::deserialize(packet)?;

        if is_child {
            let part = Part::deserialize(packet)?;
            Ok(Self::Child(part))
        } else {
            let part_id = PartId::deserialize(packet)?;
            Ok(Self::Predefined(part_id))
        }
    }
}

pub struct FreedParts(pub Box<[PartId]>);

#[derive(Resource)]
pub struct Parts {
    parts: HashMap<PartId, Part>,
    current_part_id: u32,
    ref_counts: Mutex<HashMap<PartId, usize>>,
    handle_dropped_channels: (Sender<HandleDropped>, Receiver<HandleDropped>),
}

impl Parts {
    pub fn new() -> Self {
        Self {
            parts: HashMap::new(),
            current_part_id: 0,
            ref_counts: Mutex::new(HashMap::new()),
            handle_dropped_channels: (crossbeam_channel::unbounded()),
        }
    }

    pub fn add(&mut self, part: Part) -> PartHandle {
        let id = self.insert(part);
        self.ref_counts.lock().unwrap().insert(id, 1);
        PartHandle { id, channel: self.handle_dropped_channels.0.clone() }
    }

    pub fn add_static(&mut self, part: Part) -> PartHandle {
        let id = self.insert(part);
        self.ref_counts.lock().unwrap().insert(id, 2);
        PartHandle { id, channel: self.handle_dropped_channels.0.clone() }
    }

    pub fn get_handle(&self, part_id: PartId) -> PartHandle {
        *self.ref_counts.lock().unwrap().entry(part_id).or_insert(0) += 1;
        PartHandle { id: part_id, channel: self.handle_dropped_channels.0.clone() }
    }

    pub fn get_part_from_id(&self, part_id: PartId) -> Option<&Part> {
        self.parts.get(&part_id)
    }

    pub fn get(&self, part_handle: &PartHandle) -> Option<&Part> {
        self.parts.get(&part_handle.id)
    }

    pub fn get_mut(&mut self, part_handle: &mut PartHandle) -> Option<&mut Part> {
        match self.is_child_part(part_handle) {
            Some(is_child) => {
                if is_child {
                    return self.parts.get_mut(&part_handle.id);
                }
            },
            None => { return None; }
        }

        let new_part = self.clone_part_from_part_id(part_handle.id);
        let id = self.insert(new_part);

        part_handle.id = id;
        self.ref_counts.lock().unwrap().insert(id, 1);

        self.parts.get_mut(&id)
    }

    pub fn clone_part_from_part_id(&self, parent_part_id: PartId) -> Part {
        let parent_part = self.parts.get(&parent_part_id).unwrap();
        parent_part.clone_as_child_of(parent_part_id)
    }

    fn insert(&mut self, new_part: Part) -> PartId {
        let id = PartId::from(self.current_part_id);
        self.parts.insert(id, new_part);
        self.current_part_id += 1;
        id
    }

    fn is_child_part(&self, part_handle: &PartHandle) -> Option<bool> {
        let part = self.parts.get(&part_handle.id)?;
        Some(part.parent_part_id.is_some())
    }

    fn get_unused_part_ids(&mut self) -> Box<[PartId]> {
        let mut ref_counts = self.ref_counts.lock().unwrap();
        let mut unused_part_ids = Vec::new();

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
                unused_part_ids.push(handle_dropped.0);
                ref_counts.remove(&handle_dropped.0);
            }
        }

        unused_part_ids.into_boxed_slice()
    }

}

pub fn free_parts(mut parts: ResMut<Parts>, mut freed_parts_writer: EventWriter<FreedParts>) {
    let unused_part_ids = parts.get_unused_part_ids();

    for id in unused_part_ids.iter() {
        parts.parts.remove(id);
        debug!("Removed part with ID {:?}", id);
    }

    freed_parts_writer.send(FreedParts(unused_part_ids));
}

pub struct DeletePart(pub Entity);

impl Command for DeletePart {
    fn write(self, world: &mut World) {
        let construct = world.get::<Parent>(self.0).unwrap().get();
    
        // Remove colliders
        let children = world.get::<Children>(construct).unwrap().to_vec();
        for child in children {
            if let Some(part_collider) = world.get::<PartCollider>(child) {
                if part_collider.part == self.0 {
                    world.entity_mut(construct).remove_children(&[child]);
                    world.entity_mut(child).despawn();
                }
            }
        }

        world.entity_mut(construct).remove_children(&[self.0]);
        world.entity_mut(self.0).despawn();
    }
}

pub struct PartPlugin;

impl Plugin for PartPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Parts::new())
            .insert_resource(materials::MaterialResistances::new())
            .add_event::<PlacePartRequest>()
            .add_event::<PlacePartCommand>()
            .add_event::<DeletePartRequest>()
            .add_event::<DeletePartCommand>()
            .add_event::<VoxelUpdate>()
            .add_event::<FreedParts>()
            .add_event::<RegenerateColliders>()
            .add_system(free_parts)
            .add_system(remove_unused_colliders);
    }
}