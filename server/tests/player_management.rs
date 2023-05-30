use bevy::prelude::*;
use common::player::PlayerId;
use scaffolding::{ServerTest, FixedUpdate};
use ship_designer_server::server_state::ServerState;
use uflow::client::{Client, Config};

mod scaffolding;

#[test]
fn connecting_player_gets_created() {
    let mut app = App::server_test();

    app.update();

    let mut server_address = app.world.get_non_send_resource_mut::<ServerState>().unwrap().server.address();
    server_address.set_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    
    let mut client = Client::connect(server_address, Config::default()).expect("Failed to connect to server!");
    app.fixed_update();
    let _ = client.step();
    
    app.fixed_update();

    let mut player_id_query = app.world.query::<&PlayerId>();
    assert!(player_id_query.iter(&mut app.world).len() > 0);
}

#[test]
fn disconnecting_player_gets_removed() {
    let mut app = App::server_test();

    app.update();

    let mut server_address = app.world.get_non_send_resource_mut::<ServerState>().unwrap().server.address();
    server_address.set_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    
    let mut client = Client::connect(server_address, Config::default()).expect("Failed to connect to server!");
    app.fixed_update();
    let _ = client.step();
    
    app.fixed_update();

    client.disconnect();
    let _ = client.step();
    app.fixed_update();

    let mut player_id_query = app.world.query::<&PlayerId>();
    assert!(player_id_query.iter(&mut app.world).len() == 0);
}