use bevy::prelude::*;

use common::channels::Channel;
use common::entity_lookup::lookup;
use common::fixed_update::AddFixedEvent;
use packets::Packet;
use common::part::events::{VoxelUpdate, PlacePartRequest, DeletePartRequest, PlacePartCommand, DeletePartCommand};
use common::network_id::NetworkId;
use common::part::{PartHandle, Parts, DeletePart};
use common::part::colliders::{PartCollider, RegenerateColliders, generate_collider_data};

use meshes::{PartMeshHandles, get_mesh_or_generate, free_part_mesh_handles};
use meshes::mesh_generation::{RegeneratePartMesh, regenerate_part_mesh};
use uflow::SendMode;
use crate::building_material::BuildingMaterial;
use crate::connection_state::ConnectionState;
use crate::raycast_selection::Selectable;

pub mod meshes;

pub fn spawn_part(
    commands: &mut Commands,
    mesh_handles: &mut PartMeshHandles,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<BuildingMaterial>,
    parts: &Parts,
    part_handle: PartHandle,
    transform: Transform,
    part_network_id: NetworkId,
    construct: Entity,
) -> Entity {
    let part = parts.get(&part_handle).unwrap();

    let mesh_handle = get_mesh_or_generate(part_handle.id(), part, mesh_handles, meshes);

    let part_entity = commands.spawn(MaterialMeshBundle::<BuildingMaterial> {
            mesh: mesh_handle,
            material: materials.add(BuildingMaterial { color: Color::rgb(0.0, 0.3, 0.5).into() }),
            transform,
            ..Default::default()
        })
        .insert(part_handle)
        .insert(part_network_id)
        .id();
    
    commands.entity(construct).add_child(part_entity);

    let colliders = generate_collider_data(part, transform);
    for collider_data in colliders {
        let collider_entity = commands.spawn(collider_data.collider)
            .insert(TransformBundle::from_transform(collider_data.transform))
            .insert(PartCollider::new(part_entity))
            .insert(Selectable)
            .id();
        commands.entity(construct).add_child(collider_entity);
    }

    part_entity
}

fn update_voxels(
    mut voxel_update_reader: EventReader<VoxelUpdate>,
    mut regenerate_part_mesh_writer: EventWriter<RegeneratePartMesh>,
    mut regenerate_colliders_writer: EventWriter<RegenerateColliders>,
    entity_query: Query<(Entity, &NetworkId), With<PartHandle>>,
    mut part_handle_query: Query<&mut PartHandle>,
    mut parts: ResMut<Parts>
) {
    for voxel_update in voxel_update_reader.iter() {
        if let Some(entity) = lookup(&entity_query, &voxel_update.network_id) {
            if let Ok(mut part_handle) = part_handle_query.get_mut(entity) {
                if let Some(part) = parts.get_mut(&mut part_handle) {
                    part.set_voxels(&voxel_update.voxels);
                    regenerate_part_mesh_writer.send(RegeneratePartMesh(entity));
                    regenerate_colliders_writer.send(RegenerateColliders(entity));
                }
            }
        }
    }
}

fn regenerate_colliders(
    mut commands: Commands,
    mut regenerate_colliders_reader: EventReader<RegenerateColliders>,
    parent_query: Query<&Parent, With<PartHandle>>,
    children_query: Query<&Children>,
    part_colliders_query: Query<&PartCollider>,
    part_query: Query<(&PartHandle, &Transform)>,
    parts: Res<Parts>
) {
    for request in regenerate_colliders_reader.iter() {
        let (part_handle, transform) = part_query.get(request.0).unwrap();
        let part = parts.get(&part_handle).unwrap();

        if let Ok(parent) = parent_query.get(request.0) {
            let construct = parent.get();

            // Delete old colliders
            for &child in children_query.get(construct).unwrap() {
                if let Ok(part_collider) = part_colliders_query.get(child) {
                    if part_collider.part == request.0 {
                        commands.entity(construct).remove_children(&[child]);
                        commands.entity(child).despawn();
                    }
                }
            }

            // Spawn new colliders
            let colliders = generate_collider_data(part, *transform);
            for collider_data in colliders {
                let collider_entity = commands.spawn(collider_data.collider)
                    .insert(TransformBundle::from_transform(collider_data.transform))
                    .insert(PartCollider::new(request.0))
                    .insert(Selectable)
                    .id();
                commands.entity(construct).add_child(collider_entity);
            }
        }
    }
}

fn send_place_part_requests(
    mut connection_state: ResMut<ConnectionState>,
    mut place_part_request_reader: EventReader<PlacePartRequest>
) {
    for place_part_request in place_part_request_reader.iter() {
        let packet: Packet = place_part_request.into();
        connection_state.client.send((&packet).into(), Channel::PartCommands.into(), SendMode::Reliable);
    }
}

fn send_delete_part_requests(
    mut connection_state: ResMut<ConnectionState>,
    mut delete_part_request_reader: EventReader<DeletePartRequest>
) {
    for delete_part_request in delete_part_request_reader.iter() {
        let packet: Packet = delete_part_request.into();
        connection_state.client.send((&packet).into(), Channel::PartCommands.into(), SendMode::Reliable);
    }
}

fn place_parts(
    mut place_part_command_reader: EventReader<PlacePartCommand>,
    mut commands: Commands,
    mut mesh_handles: ResMut<PartMeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BuildingMaterial>>,
    parts: Res<Parts>,
    entity_query: Query<(Entity, &NetworkId)>,
) {
    for event in place_part_command_reader.iter() {
        let transform = Transform::from(event.transform);
        let entity = spawn_part(
            &mut commands,
            &mut mesh_handles,
            &mut meshes,
            &mut materials,
            &parts,
            parts.get_handle(event.part_id),
            transform,
            event.part_network_id,
            lookup(&entity_query, &event.construct_network_id).unwrap()
        );
        
        debug!("Spawned part with entity ID {:?}", entity);
    }
}

fn delete_parts(
    mut delete_part_command_reader: EventReader<DeletePartCommand>,
    mut commands: Commands,
    part_query: Query<(Entity, &NetworkId)>
) {
    for event in delete_part_command_reader.iter() {
        if let Some(part) = lookup(&part_query, &event.0) {
            commands.add(DeletePart(part));
        }
    }
}

pub struct ClientPartPlugin;

impl Plugin for ClientPartPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PartMeshHandles::new())
            .add_fixed_event::<RegeneratePartMesh>()
            .add_fixed_event::<VoxelUpdate>()
            .add_systems((
                update_voxels,
                regenerate_part_mesh.after(update_voxels),
                regenerate_colliders.after(update_voxels),
                free_part_mesh_handles,
                send_place_part_requests,
                send_delete_part_requests,
                place_parts,
                delete_parts,
            ).in_schedule(CoreSchedule::FixedUpdate));
    }
}