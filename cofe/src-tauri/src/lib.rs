use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use keyring::Entry;
use once_cell::sync::OnceCell;
use libp2p::{
    futures::StreamExt,
    identity,
    PeerId,
};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tokio::sync::{RwLock, mpsc};
use tracing::{warn};

use crate::config::Config;
use crate::message::chat::chat_store::{chat_dir, load_chat, save_chat};
use crate::security::security::{ derive_storage_key, load_salt_from_disk, load_storage_key, save_salt_to_disk, save_storage_key};
use crate::node_identity::identity::{ load_or_create_identity};
use crate::node_identity::peers::load_peers_from_disk;
use crate::p2p::command::{P2PCommand, handle_command};
use crate::p2p::connection_p2p::P2P;
use crate::p2p::event::{P2PEvent, handle_swarm_event};

// Crate
mod message;
mod node_identity;
mod p2p;
mod security;
mod config;

static APP_DATA_DIR: OnceCell<PathBuf> = OnceCell::new();
const SERVICE: &str = "vanadinite-chat";
const KEY_NAME: &str = "storage-key";


// Struct
struct AppState {
    pub identity: identity::Keypair,
    pub tx: mpsc::Sender<P2PCommand>,
    pub peer_store: Arc<PeerStore>,
    pub entry: Entry
}

struct CredentialState {
    pub storage_key: Option<[u8; 32]>,
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

struct P2PService {
    event_tx: mpsc::Sender<P2PEvent>,
}



// Function / Tools

async fn on_message_received(
    app: &tauri::AppHandle,
    peer: PeerId,
    msg: ChatMessage,
) {
    let cred = app.state::<CredentialState>();

    let Some(storage_key) = cred.storage_key.as_ref() else {
        warn!("Message received but app is locked, message skipped");
        return;
    };


    let mut chats = load_chat(
        &peer.to_string(),
        storage_key,
        chat_dir(),
    ).unwrap_or_default();

    chats.push(msg.clone());

    save_chat(
        peer.to_string(),
        &chats,
        storage_key,
        chat_dir(),
    ).ok();

    app.emit("message-received", (peer.to_string(), msg)).ok();
}


fn start(app: &tauri::AppHandle) {
    let cfg = Config::load().unwrap();
    let local_key = load_or_create_identity();
    let entry = Entry::new(SERVICE, KEY_NAME).unwrap();
    // let storage_key = derive_storage_key(&local_key);
    
    let mut p2p = P2P::new(local_key.clone(), cfg).expect("failed to create P2P");;
    
    let (tx, mut rx) = mpsc::channel::<P2PCommand>(32);
    let (event_tx, mut event_rx) = mpsc::channel::<P2PEvent>(32);
    let ctx = P2PService{event_tx};


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
                   on_message_received(&app_handle, peer, msg).await;
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


// Command


#[tauri::command]
fn load_config()  -> Result<Config, String> {
    Config::load().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config(cfg: Config) -> Result<(), String> {
    cfg.validate()?;
    cfg.save().map_err(|e| e.to_string())
}



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

    let cred = app.state::<CredentialState>();
    let message_state = app.state::<AppState>();

    let mut chats = load_chat(
        &peer_id,
        cred.storage_key.as_ref().ok_or("App locked")?,
       chat_dir(),
    ).unwrap_or_default();  

    let _ = message_state.tx.send(P2PCommand::SendChat { peer: peer_id.clone().parse::<PeerId>().unwrap(), msg: message.clone() }).await;
    log::info!("send message");
    chats.push(message);

    save_chat(
        peer_id.clone(),
        &chats,
        cred.storage_key.as_ref().ok_or("App locked")?,
        chat_dir(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
fn setup_password(app: tauri::AppHandle, state: tauri::State<'_, AppState>, password: String) -> Result<(), String> {
    let salt = rand::random::<[u8; 16]>();

    let storage_key = derive_storage_key(&password, &salt);

    save_storage_key(&storage_key, &state.entry);
    let _ = save_salt_to_disk(&app, &salt);
    app.manage(CredentialState{storage_key: Some(storage_key)});

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

    app.manage(CredentialState{storage_key: Some(derived)});

    app.emit("app-ready", ()).ok();
    Ok(())
}

#[tauri::command]
fn get_self_peer_id(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.identity.public().to_peer_id().to_string())
}

#[tauri::command]
fn get_history_message(app: tauri::AppHandle, peer_id: String) -> Result<Vec<ChatMessage>, String> {
    let cred = app.state::<CredentialState>();
    let storage_key = cred.storage_key.as_ref().ok_or("App locked")?;

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
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            send_greet, 
            find_peer, 
            send_message, 
            setup_password, 
            unlock_app, 
            get_self_peer_id, 
            get_history_message, 
            get_first_run
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
