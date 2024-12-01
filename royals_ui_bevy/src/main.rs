use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_renet::{
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport},
        ConnectionConfig, DefaultChannel, RenetClient,
    },
    transport::NetcodeClientPlugin,
    RenetClientPlugin,
};
use royals_core::{events::GameEvent, user_name::Username};
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};
use ui::{ui_system, ClientEventComponent};

pub mod ui;

fn main() {
    let server_addr: SocketAddr = "127.0.0.1:6969".parse().unwrap();
    let username = Username::from_string("bevy".to_string());
    let connection_config = ConnectionConfig::default();
    let client = RenetClient::new(connection_config);

    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        server_addr,
        client_id,
        user_data: Some(username.to_netcode_user_data()),
        protocol_id: 0,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    App::new()
        // -----------------------------------------------------
        .add_plugins(RenetClientPlugin)
        .add_plugins(NetcodeClientPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        // -----------------------------------------------------
        .insert_resource(client)
        .insert_resource(transport)
        .insert_resource(GameState { last_event: None })
        // -----------------------------------------------------
        .add_systems(Update, send_message_system)
        .add_systems(Update, receive_message_system)
        .add_systems(Update, ui_system)
        // -----------------------------------------------------
        .run();
}

#[derive(Resource)]
pub struct GameState {
    pub last_event: Option<GameEvent>,
}

fn send_message_system(
    mut commands: Commands,
    query: Query<(Entity, &ClientEventComponent)>,
    mut client: ResMut<RenetClient>,
) {
    for (entity, client_event) in query.iter() {
        let text = serde_json::to_string(&client_event.e).unwrap();
        client.send_message(DefaultChannel::ReliableOrdered, text.as_bytes().to_vec());
        commands.entity(entity).despawn();
    }
}

fn receive_message_system(mut client: ResMut<RenetClient>, mut game_state: ResMut<GameState>) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        let message = String::from_utf8(message.into()).unwrap();
        if let Ok(event) = serde_json::from_str::<GameEvent>(&message) {
            game_state.as_mut().last_event = Some(event);
        }
    }
}
