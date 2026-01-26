use std::{fs, path::PathBuf};

use crate::{APP_DATA_DIR, ChatMessage, security::security::{decrypt, encrypt}};

pub fn save_chat(
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

pub fn load_chat(
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

pub fn chat_dir() -> PathBuf {
    let mut dir = APP_DATA_DIR.get().expect("app dir not initialized").clone();
    dir.push("chats");
    std::fs::create_dir_all(&dir).ok();
    dir
}
