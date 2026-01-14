use std::{fs, path::PathBuf};

use aes_gcm::aead::Aead;
use argon2::Argon2;
use keyring::{Entry};
use tauri::Manager;


pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = rand::random::<[u8; 12]>();
    let encrypted = cipher.encrypt(Nonce::from_slice(&nonce), data).unwrap();

    [nonce.to_vec(), encrypted].concat()
}

pub fn decrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    let (nonce, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();

    cipher.decrypt(Nonce::from_slice(nonce), ciphertext).unwrap()
}

pub fn derive_storage_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];

    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .unwrap();

    output
}

pub fn save_storage_key(key: &[u8; 32], entry: &Entry) {
    entry.set_password(&base64::encode(key)).unwrap();

    log::info!("password: {}", entry.get_password().unwrap());
}

pub fn load_storage_key(entry: &Entry) -> Option<[u8; 32]> {
    log::info!("load_storage_key start");

    let encoded = match entry.get_password() {
        Ok(v) => {
            log::info!("password loaded from keyring");
            v
        }
        Err(e) => {
            log::warn!("failed to load password from keyring: {e}");
            return None;
        }
    };

    let bytes = base64::decode(&encoded).ok()?;

    if bytes.len() != 32 {
        log::error!("invalid key length: {}", bytes.len());
        return None;
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Some(key)
}


pub fn save_salt_to_disk(
    app: &tauri::AppHandle,
    salt: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut path: PathBuf = app.path()
        .app_data_dir()
        .expect("cannot resolve app data dir");

    fs::create_dir_all(&path)?;

    path.push("salt.bin");

    fs::write(path, salt)?;

    Ok(())
}

pub fn load_salt_from_disk(
    app: &tauri::AppHandle,
) -> Option<Vec<u8>> {
    let mut path = app.path().app_data_dir().ok()?;
    path.push("salt.bin");

    fs::read(path).ok()
}
