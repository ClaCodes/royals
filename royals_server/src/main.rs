mod game_lobby;
mod game_logic;
mod player;
mod random_playing_computer;
mod remote_player;
mod utils;

use crate::{
    player::Player, random_playing_computer::RandomPlayingComputer, remote_player::RemotePlayer,
};
use game_lobby::GameLobby;
use renet::{
    transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    ClientId, ConnectionConfig, DefaultChannel, RenetServer, ServerEvent,
};
use royals_core::{
    events::{ClientEvent, GameEvent},
    user_name::Username,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant, SystemTime},
};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time,
};

#[derive(Deserialize, Serialize)]
struct ServerEvent2 {
    id: ClientId,
    event: GameEvent,
}

pub fn run_game<C, T>(player_constructor: C)
where
    C: FnOnce() -> T,
    T: Player + 'static,
{
    let mut lobby = GameLobby::new();
    lobby.add_player(player_constructor);
    lobby.add_player(RandomPlayingComputer::new);
    lobby.add_player(RandomPlayingComputer::new);
    lobby.add_player(RandomPlayingComputer::new);
    lobby.play_round();
}

async fn game_loop(
    name: String,
    sender: std::sync::mpsc::Sender<GameEvent>,
    receiver: std::sync::mpsc::Receiver<usize>,
) {
    run_game(move || RemotePlayer::new(name, sender, receiver));
}

async fn serve(
    username: Username,
    id: ClientId,
    mut client_event: Receiver<ClientEvent>,
    server_event: Sender<ServerEvent2>,
) {
    let (sender1, receiver1) = std::sync::mpsc::channel();
    let (sender2, receiver2) = std::sync::mpsc::channel();
    tokio::spawn(game_loop(username.to_str(), sender1, receiver2));
    let mut interval = time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Ok(event) = receiver1.try_recv() {
                    server_event.send(ServerEvent2{event,id}).await.unwrap();
                }
            }
            msg = client_event.recv() => {
                if let Some(data) = msg {
                    sender2.send(data.action_id).unwrap();
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let mut interval = time::interval(Duration::from_millis(50));
    env_logger::init();
    println!("Usage: [SERVER_PORT]");
    let args: Vec<String> = std::env::args().collect();
    let public_addr: SocketAddr = format!("0.0.0.0:{}", args[1]).parse().unwrap();
    let connection_config = ConnectionConfig::default();
    let mut server: RenetServer = RenetServer::new(connection_config);

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: 0,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };
    let socket: UdpSocket = UdpSocket::bind(public_addr).unwrap();

    let mut transport = NetcodeServerTransport::new(server_config, socket).unwrap();

    let mut client_channels: HashMap<ClientId, Sender<ClientEvent>> = HashMap::new();

    let mut last_updated = Instant::now();
    let (server_event_tx, mut server_event_rx) = channel::<ServerEvent2>(1);

    loop {
        let now = Instant::now();
        let duration = now - last_updated;
        last_updated = now;

        server.update(duration);
        transport.update(duration, &mut server).unwrap();

        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    let user_data = transport.user_data(client_id).unwrap();
                    let username = Username::from_user_data(&user_data);
                    println!("Client {} connected.", username.to_str());
                    let (client_event_tx, client_event_rx) = channel::<ClientEvent>(1);
                    client_channels.insert(client_id, client_event_tx.clone());
                    tokio::spawn(serve(
                        username,
                        client_id,
                        client_event_rx,
                        server_event_tx.clone(),
                    ));
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    println!("Client {} disconnected: {}", client_id, reason);
                    if let Some(_client) = client_channels.remove(&client_id) {}
                }
            }
        }

        for client_id in server.clients_id() {
            while let Some(message) =
                server.receive_message(client_id, DefaultChannel::ReliableOrdered)
            {
                let text = String::from_utf8(message.into()).unwrap();
                if let Ok(msg) = serde_json::from_str(&text) {
                    let client_channel = client_channels.get(&client_id).unwrap();
                    _ = client_channel.send(msg).await;
                }
            }
        }

        transport.send_packets(&mut server);
        tokio::select! {
            _ = interval.tick() => {}
            data = server_event_rx.recv() => {
                if let Some(msg) = data {
                    if let Ok(s) = serde_json::to_string(&msg.event) {
                        server.send_message(msg.id, DefaultChannel::ReliableOrdered, s);
                    }
                }
            }
        }
    }
}
