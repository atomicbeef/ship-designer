use bevy::prelude::*;

use common::part::events::{PlacePartRequest, PlacePartCommand};
use common::compact_transform::CompactTransform;
use common::ship::ShipBundle;
use scaffolding::{ServerTest, FixedUpdate};
use ship_designer_server::network_id_generator::NetworkIdGenerator;

mod scaffolding;

#[test]
fn cannot_place_overlapping_parts() {
    let mut app = App::server_test();

    let construct_network_id = app.world.get_resource_mut::<NetworkIdGenerator>().unwrap().generate();
    app.world.spawn(ShipBundle {
        transform: TransformBundle::from_transform(Transform {
            translation: Vec3::splat(0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(1.0)
        }),
        network_id: construct_network_id,
        ..Default::default()
    });

    let place_part_request = PlacePartRequest {
        part_id: 0.into(),
        part_transform: CompactTransform::from(Transform::from_xyz(0.0, 0.0, 0.0)),
        construct_network_id: construct_network_id,
    };

    app.world
        .get_resource_mut::<Events<PlacePartRequest>>()
        .unwrap()
        .send(place_part_request.clone());
    app.world
        .get_resource_mut::<Events<PlacePartRequest>>()
        .unwrap()
        .send(place_part_request);

    app.fixed_update();

    assert_eq!(app.world.get_resource::<Events<PlacePartCommand>>().unwrap().len(), 1);
}

#[test]
fn can_place_non_overlapping_parts() {
    let mut app = App::server_test();

    let construct_network_id = app.world.get_resource_mut::<NetworkIdGenerator>().unwrap().generate();
    app.world.spawn(ShipBundle {
        transform: TransformBundle::from_transform(Transform {
            translation: Vec3::splat(0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(1.0)
        }),
        network_id: construct_network_id,
        ..Default::default()
    });

    let place_part_request_1 = PlacePartRequest {
        part_id: 0.into(),
        part_transform: CompactTransform::from(Transform::from_xyz(0.0, 0.0, 0.0)),
        construct_network_id: construct_network_id,
    };
    let place_part_request_2 = PlacePartRequest {
        part_transform: CompactTransform::from(Transform::from_xyz(100.0, 50.0, 50.0)),
        ..place_part_request_1
    };

    app.world
        .get_resource_mut::<Events<PlacePartRequest>>()
        .unwrap()
        .send(place_part_request_1);
    app.world
        .get_resource_mut::<Events<PlacePartRequest>>()
        .unwrap()
        .send(place_part_request_2);

    app.fixed_update();

    assert_eq!(app.world.get_resource::<Events<PlacePartCommand>>().unwrap().len(), 2);
}