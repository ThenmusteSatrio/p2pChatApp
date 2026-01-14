use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use keyring::Entry;
use once_cell::sync::OnceCell;

use libp2p::kad;
use libp2p::{
    core::{
        transport::{upgrade, OrTransport},
        Transport,
    },
    futures::StreamExt,
    identify, identity, noise, request_response,
    swarm::SwarmEvent,
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn};

use crate::message::security::{decrypt, derive_storage_key, encrypt, load_salt_from_disk, load_storage_key, save_salt_to_disk, save_storage_key};
use crate::node_identity::identity::{ load_or_create_identity};
use crate::node_identity::peers::load_peers_from_disk;
use crate::{
    message::message::{GreetRequest, GreetResponse},
    p2p::{
        behaviour::{Behaviour as AgentBehaviour, Event as AgentEvent},
    },
};

// Crate
mod message;
mod node_identity;
mod p2p;

static APP_DATA_DIR: OnceCell<PathBuf> = OnceCell::new();
static STORAGE_KEY: OnceCell<[u8; 32]> = OnceCell::new();
const SERVICE: &str = "vanadinite-chat";
const KEY_NAME: &str = "storage-key";


// Struct
struct AppState {
    pub identity: identity::Keypair,
    pub tx: mpsc::Sender<P2PCommand>,
    pub peer_store: Arc<PeerStore>,
    pub entry: Entry
}
struct CredentialState{
    pub storage_key: [u8; 32],
}
pub struct P2P {
    local_key: identity::Keypair,
    bootstrap_node_addr: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StoredPeer {
    peer_id: String,
    addrs: Vec<String>,
    last_seen: i64,
    success: u32,
    fail: u32,
}

pub struct PeerStore{
    pub peers: RwLock<HashMap<PeerId, StoredPeer>>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    id: String,
    from: PeerId,
    to: PeerId,
    timestamp: i64,
    content: String,
    // direction: MessageDirection,
}
struct ChatThread {
    peer: PeerId,
    messages: Vec<ChatMessage>,
}
struct P2PContext {
    event_tx: mpsc::Sender<P2PEvent>,
}



// Enum

enum P2PCommand {
    SendGreet { peer: PeerId, msg: String },
    SendChat { peer: PeerId, msg: ChatMessage },
    FindNode { peer: PeerId },
}
enum P2PEvent {
    PeerDiscovered(PeerId),
    MessageReceived { peer: PeerId, msg: ChatMessage },
}

// #[derive(Serialize, Deserialize, Clone, Debug)]
// enum MessageDirection {
//     Incoming,
//     Outgoing,
// }

// Impl

impl P2P {
    pub fn new(local_key: identity::Keypair) -> Self {
        Self {
            local_key: local_key,
            bootstrap_node_addr: Some(
                "12D3KooWLjEhL6g7krca9QG5pDhir6CXbCZHmNahnGb6DNUyssro".to_string(),
            ),
        }
    }

    async fn create_p2p(&mut self) -> Result<Swarm<AgentBehaviour>, Box<dyn std::error::Error>> {
        let mut swarm = SwarmBuilder::with_existing_identity(self.local_key.clone())
            .with_tokio()
            .with_other_transport(move |keypair| {
                let tcp_transport: libp2p::tcp::Transport<libp2p::tcp::tokio::Tcp> =
                    tcp::Transport::new(tcp::Config::default().nodelay(true));
                let ws_transport = libp2p::websocket::Config::new(tcp::tokio::Transport::new(
                    tcp::Config::default(),
                ));

                OrTransport::new(ws_transport, tcp_transport)
                    .upgrade(upgrade::Version::V1)
                    .authenticate(noise::Config::new(&keypair).unwrap())
                    .multiplex(yamux::Config::default())
                    .boxed()
            })?
            .with_behaviour(|keypair| {
                let local_peer_id = PeerId::from(keypair.clone().public());

                let kad_config = kad::Config::default();
                let kad_memory = kad::store::MemoryStore::new(local_peer_id);
                let kad = kad::Behaviour::with_config(local_peer_id, kad_memory, kad_config);

                let identify_config = identify::Config::new(
                    "/agent/connection/1.0.0".to_string(),
                    keypair.clone().public(),
                )
                .with_push_listen_addr_updates(true)
                .with_interval(Duration::from_secs(30));
                let identify = identify::Behaviour::new(identify_config);

                let rr_config = request_response::Config::default();
                let rr_protocol = StreamProtocol::new("/agent/message/1.0.0");
                let rr_behavior =
                    request_response::cbor::Behaviour::<GreetRequest, GreetResponse>::new(
                        [(rr_protocol, request_response::ProtocolSupport::Full)],
                        rr_config,
                    );

                AgentBehaviour {
                    identify: identify,
                    kad: kad,
                    rr: rr_behavior,
                }
            })?
            .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(30)))
            .build();

        swarm
            .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
            .unwrap();

        let peer_id = self
            .bootstrap_node_addr
            .clone()
            .unwrap()
            .parse::<PeerId>()
            .map_err(|_| "Peer ID is invalid")
            .unwrap();
        let remote: Multiaddr = format!("/ip4/127.0.0.1/tcp/8000/ws/p2p/{}", peer_id).parse()?;

        swarm.dial(remote)?;

        Ok(swarm)
    }
}

// Function / Tools
async fn handle_swarm_event(event: SwarmEvent<AgentEvent>, swarm: &mut Swarm<AgentBehaviour>, peer_store: &PeerStore, event_tx: &mpsc::Sender<P2PEvent>) {
    match event {
         SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => info!("NewListenAddr: {listener_id:?} | {address:?}"),
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            } => info!("ConnectionEstablished: {peer_id:?} | {connection_id:?} | {endpoint:?} | {num_established:?} | {concurrent_dial_errors:?} | {established_in:?}"),
            SwarmEvent::Dialing { peer_id, connection_id } => info!("Dialing: {peer_id:?} | {connection_id}"),
            SwarmEvent::Behaviour(AgentEvent::Identify(event)) => match event {
                identify::Event::Sent { connection_id, peer_id } => info!("Sent: {connection_id} | {peer_id}"),
                identify::Event::Pushed { connection_id, peer_id, info } => info!("Pushed: {connection_id} | {peer_id} | {info:?}"),
                //^ menerima informasi identitas (peer lain) dari jaringan
                identify::Event::Received { connection_id: _, peer_id, info } => {
                    let mut peers = peer_store.peers.write().await;

                    let entry = peers.entry(peer_id).or_insert(StoredPeer { peer_id: peer_id.to_string(), addrs: vec![], last_seen: chrono::Utc::now().timestamp(), success: 0, fail: 0 });
                    entry.addrs = info.listen_addrs.iter().map(|a| a.to_string()).collect();

                    entry.last_seen = chrono::Utc::now().timestamp();
                },
                _ => {}
            }
            SwarmEvent::Behaviour(AgentEvent::RequestResponse(event)) => match event {
                request_response::Event::Message { peer, connection_id:_, message } => {
                    match message {
                        request_response::Message::Request { request_id, request, channel } => {
                            info!("request_response::Event::Message::Request -> PeerID: {peer} | RequestID: {request_id} | RequestMessage: {request:?}");
                            match request {
                                GreetRequest::Syn { message } => {}
                                GreetRequest::Chat { message } => {
                                    let storage_key = STORAGE_KEY.get().expect("storage key not ready");
                                    log::info!("storage key: {storage_key:?}");
                                    let mut chats = load_chat(
                                        &peer.to_string(),
                                        &storage_key,
                                        chat_dir(),
                                    ).unwrap_or_default();
                                    log::info!("chats: {chats:?}");
                                    log::info!("message: {message:?}");
                                    chats.push(message.clone());

                                    let _ = save_chat(
                                        peer.to_string(),
                                        &chats,
                                        &storage_key,
                                        chat_dir(),
                                    );
                                    
                                    let saved_chats = load_chat(
                                        &peer.to_string(),
                                        &storage_key,
                                    chat_dir(),
                                    ).unwrap_or_default();  
                                    
                                    log::info!("saved chats: {saved_chats:?}");
                                    let _ = event_tx.send(P2PEvent::MessageReceived { peer, msg: message }).await;
                                }
                            }
                        }
                        
                        request_response::Message::Response { request_id, response } => {
                            info!(" request_response::Event::Message::Response -> PeerID: {peer} | RequestID: {request_id} | ResponseMessage: {response:?}");
                            match response {
                                GreetResponse::Ack { message } => {}
                            }
                        }
                    }
                }
                request_response::Event::InboundFailure { peer, connection_id, request_id, error } => {
                    warn!("request_response::Event::InboundFailure -> PeerID: {peer} | ConnectionID: {connection_id} | RequestID: {request_id} | Error: {error:?}")
                }
                request_response::Event::OutboundFailure { peer, connection_id, request_id, error } => {
                    warn!("request_response::Event::OutboundFailure -> PeerID: {peer} | ConnectionID: {connection_id} | RequestID: {request_id} | Error: {error:?}")
                }
                request_response::Event::ResponseSent { peer, connection_id, request_id } => {
                    info!("request_response::Event::ResponseSent -> PeerID: {peer} | ConnectionID: {connection_id} | RequestID: {request_id}")
                }
            }
            //@ Event dipicu ketika ada query
            SwarmEvent::Behaviour(AgentEvent::Kad(kad::Event::OutboundQueryProgressed { id, result, stats: _, step:_ })) => {
                if let kad::QueryResult::GetClosestPeers(Ok(ok)) = result {
                    
                }
            }
        
            SwarmEvent::Behaviour(AgentEvent::Kad(event)) => match event {
                kad::Event::ModeChanged { new_mode }=> {
                    info!("ModeChanged: {new_mode:?}")
                }
                kad::Event::RoutablePeer { peer, address } => {
                    info!("kad::Event::RoutablePeer -> PeerID: {peer} | Address: {address}")
                }
                kad::Event::PendingRoutablePeer { peer, address } => {
                    info!("kad::Event::PendingRoutablePeer -> PeerID: {peer} | Address: {address}")
                }
                kad::Event::InboundRequest { request } => {
                    info!("kad::Event::InboundRequest -> Request: {request:?}")
                }
                kad::Event::RoutingUpdated { peer, is_new_peer, addresses, bucket_range, old_peer } => {
                    info!("kad::Event::RoutingUpdated: {peer} | IsNewPeer: {is_new_peer} | Addresses: {addresses:?} | BucketRange: {bucket_range:?} | OldPeer: {old_peer:?}");
                    let mut peers = peer_store.peers.write().await;

                    let entry = peers.entry(peer).or_insert(StoredPeer {
                        peer_id: peer.to_string(),
                        addrs: vec![],
                        last_seen: chrono::Utc::now().timestamp(),
                        success: 0,
                        fail: 0,
                    });

                    entry.addrs = addresses.iter().map(|a| a.to_string()).collect();
                    entry.last_seen = chrono::Utc::now().timestamp();
                }
                kad::Event::OutboundQueryProgressed { id, result, stats, step } => {
                    info!("kad::Event::OutboundQueryProgressed: ID: {id:?} | Result: {result:?} | Stats: {stats:?} | Step: {step:?}");
                }
                _ => {}
            }
            _ => {}
    }
}

fn handle_command(cmd: P2PCommand, swarm: &mut Swarm<AgentBehaviour>) {
    match cmd {
        P2PCommand::SendGreet { peer, msg } => {
            swarm
                .behaviour_mut()
                .rr
                .send_request(&peer, GreetRequest::Syn { message: msg });
        },
        P2PCommand::SendChat { peer, msg } => {
            log::info!("send chat: {msg:?}");
            swarm
                .behaviour_mut()
                .rr
                .send_request(&peer, GreetRequest::Chat{ message: msg });
        },
        P2PCommand::FindNode { peer } => {
            swarm
                .behaviour_mut()
                .kad
                .get_closest_peers(peer);
        }
    }
}

fn start(app: &tauri::AppHandle) {
    let local_key = load_or_create_identity();
    let entry = Entry::new(SERVICE, KEY_NAME).unwrap();
    // let storage_key = derive_storage_key(&local_key);
    
    let mut p2p = P2P::new(local_key.clone());
    log::info!("created p2p");
    
    let (tx, mut rx) = mpsc::channel::<P2PCommand>(32);
    let (event_tx, mut event_rx) = mpsc::channel::<P2PEvent>(32);
    let ctx = P2PContext{event_tx};


    let peer_store = Arc::new(PeerStore {
        peers: RwLock::new(HashMap::new()),
    });

    app.manage(AppState {identity: local_key.clone(), entry: entry, tx, peer_store: peer_store.clone() });

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                P2PEvent::PeerDiscovered(peer) => {
                    app_handle.emit("peer-discovered", peer.to_string()).ok();
                }
                P2PEvent::MessageReceived { peer, msg } => {
                    app_handle.emit("message-received", (peer.to_string(), msg)).ok();
                }
            }
        }
    });

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut swarm = p2p.create_p2p().await.unwrap();
        let stored_peers = load_peers_from_disk(&app_handle);
        let mut to_add = Vec::new();

        {
            let mut peers = peer_store.peers.write().await;
            for (peer_id, peer) in stored_peers {
                for addr in &peer.addrs {
                    if let Ok(addr) = addr.parse() {
                        to_add.push((peer_id, addr));
                    }
                }
                peers.insert(peer_id, peer);
            }
        }
        for (peer_id, addr) in to_add {
            swarm.behaviour_mut().kad.add_address(&peer_id, addr);
        }
        
        loop {
            tokio::select! {
                event = swarm.select_next_some() => {
                    handle_swarm_event(event, &mut swarm, &peer_store, &ctx.event_tx).await;
                }
                Some(cmd) = rx.recv() => {
                    handle_command(cmd, &mut swarm);
                }
            }
        }
    });
}

fn save_chat(
    peer_id: String,
    message: &Vec<ChatMessage>,
    key: &[u8; 32],
    base_dir: PathBuf
) -> Result<(), Box<dyn std::error::Error>>{
    let json = serde_json::to_vec(message)?;
    let encrypted = encrypt(&json, key);

    let mut path = base_dir;
    path.push(format!("{}.enc", peer_id));

    fs::write(path, encrypted)?;
    Ok(())
}

fn load_chat(
    peer_id: &str,
    key: &[u8; 32],
    base_dir: PathBuf,
) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error>> {

    let mut path = base_dir;
    path.push(format!("{}.enc", peer_id));

    if !path.exists() {
        return Ok(vec![]);
    }

    let encrypted = std::fs::read(path)?;
    let decrypted = decrypt(&encrypted, key);

    let messages = serde_json::from_slice(&decrypted)?;
    Ok(messages)
}

fn chat_dir() -> PathBuf {
    let mut dir = APP_DATA_DIR.get().expect("app dir not initialized").clone();
    dir.push("chats");
    std::fs::create_dir_all(&dir).ok();
    dir
}


// Command
#[tauri::command]
async fn send_greet(state: tauri::State<'_, AppState>, name: String) -> Result<String, String> {
    state
        .tx
        .send(P2PCommand::SendGreet {
            peer: name.parse::<PeerId>().unwrap(),
            msg: "hello".to_string(),
        })
        .await
        .unwrap();
    log::info!("send_greet");
    Ok("Success".to_string())
}

#[tauri::command]
async fn find_peer(state: tauri::State<'_, AppState>, peer_id: String) -> Result<Vec<PeerId>, String> {
    state
        .tx
        .send(P2PCommand::FindNode {
            peer: peer_id.parse::<PeerId>().unwrap(),
        })
        .await
        .unwrap();
    // log::info!("send_greet");
    log::info!("find peer: {:?}", state.peer_store.peers.read().await.keys().cloned().collect::<Vec<PeerId>>());
    Ok(state.peer_store.peers.read().await.keys().cloned().collect())
}

#[tauri::command]
async fn send_message(
    app: tauri::AppHandle,
    peer_id: String,
    message: ChatMessage,
) -> Result<(), String> {

    let state = app.state::<CredentialState>();
    let message_state = app.state::<AppState>();

    let mut chats = load_chat(
        &peer_id,
        &state.storage_key,
       chat_dir(),
    ).unwrap_or_default();  

    let _ = message_state.tx.send(P2PCommand::SendChat { peer: peer_id.clone().parse::<PeerId>().unwrap(), msg: message.clone() }).await;
    log::info!("send message");
    chats.push(message);

    save_chat(
        peer_id.clone(),
        &chats,
        &state.storage_key,
        chat_dir(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
fn setup_password(app: tauri::AppHandle, state: tauri::State<'_, AppState>, password: String) -> Result<(), String> {
    let salt = rand::random::<[u8; 16]>();

    let storage_key = derive_storage_key(&password, &salt);

    save_storage_key(&storage_key, &state.entry);
    let _ = save_salt_to_disk(&app, &salt);
    app.manage(CredentialState{storage_key: storage_key});
    STORAGE_KEY.set(storage_key).ok();

    Ok(())
}

#[tauri::command]
fn unlock_app(app: tauri::AppHandle, state: tauri::State<'_, AppState>, password: String) -> Result<(), String> {
    let salt = load_salt_from_disk(&app)
        .ok_or("Salt not found. App corrupted?")?;

    let derived = derive_storage_key(&password, &salt);

    let stored = load_storage_key(&state.entry)
        .ok_or("Storage key not found")?;

    if derived != stored {
        return Err("Invalid password".into());
    }

    STORAGE_KEY.set(derived).ok();

    app.emit("app-ready", ()).ok();
    Ok(())
}

#[tauri::command]
fn get_self_peer_id(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.identity.public().to_peer_id().to_string())
}

#[tauri::command]
fn get_history_message(app: tauri::AppHandle, peer_id: String) -> Result<Vec<ChatMessage>, String> {
    let storage_key = STORAGE_KEY.get().expect("storage key not ready");
    // log::info!("get history message: {}", peer_id);
    log::info!("storage key: {:?}", &storage_key);
    log::info!("peer id: {}", peer_id);
     let chats = load_chat(
        &peer_id,
        &*storage_key,
       chat_dir(),
    ).unwrap_or_default();  
    log::info!("get history message: {:?}", chats);
    Ok(chats)
}


#[tauri::command]
fn get_first_run(state: tauri::State<'_, AppState>)-> Result<bool, String> {
    // load_storage_key();
     if load_storage_key(&state.entry).is_none() {
        Ok(true)
    } else {
        Ok(false)
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            let dir = app
            .path()
            .app_data_dir()
            .expect("cannot resolve app data dir");

            std::fs::create_dir_all(&dir)?;
            APP_DATA_DIR.set(dir).unwrap();
                start(&app.handle());
                Ok(())
            })
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                )) 
                .build(),
        )
        .invoke_handler(tauri::generate_handler![send_greet, find_peer, send_message, setup_password, unlock_app, get_self_peer_id, get_history_message, get_first_run])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
