use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use libp2p::PeerId;
use tauri::Manager;
use tokio::fs as async_fs;

use crate::{PeerStore, StoredPeer};

async fn save_peers_to_disk(
    app: &tauri::AppHandle,
    peer_store: Arc<PeerStore>,
) {
    let path = peers_file_path(app);

    let peers = peer_store.peers.read().await;

    let raw: HashMap<String, StoredPeer> = peers
        .iter()
        .map(|(id, peer)| (id.to_string(), peer.clone()))
        .collect();

    if let Ok(json) = serde_json::to_string_pretty(&raw) {
        let _ = async_fs::write(path, json).await;
    }
}


fn peers_file_path(app: &tauri::AppHandle) -> PathBuf {
    let mut dir = app.path().resolve("p2p", tauri::path::BaseDirectory::AppData).expect("Failed to resolve app data dir");
    std::fs::create_dir_all(&dir).ok();
    dir.push("peers.json");
    dir
}

pub fn load_peers_from_disk(app: &tauri::AppHandle) -> HashMap<PeerId, StoredPeer> {
    let path = peers_file_path(app);

    if !path.exists() {
        return HashMap::new();
    }

    let data = match fs::read_to_string(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("failed to read peers file: {e}");
            return HashMap::new();
        }
    };

    let raw: HashMap<String, StoredPeer> = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("invalid peers.json: {e}");
            return HashMap::new();
        }
    };

    raw.into_iter().filter_map(|(peer_id, peer)|{
        peer_id.parse::<PeerId>().ok().map(|id| (id, peer))
    }).collect()
}