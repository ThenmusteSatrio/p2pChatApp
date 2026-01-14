use libp2p::identity::{self, ed25519::SecretKey};
use schnorrkel::MiniSecretKey;
use keyring::Entry;
use hex;
// use ed25519_dalek::SecretKey;
use x25519_dalek::{StaticSecret, PublicKey};

pub fn generate_ed25519_key_id() -> (libp2p::identity::Keypair, schnorrkel::Keypair) {
    let ed_key = identity::ed25519::Keypair::generate();
    let local_key: identity::Keypair = identity::Keypair::from(ed_key.clone());

    let secret = ed_key.secret();

    let mini = MiniSecretKey::from_bytes(secret.as_ref()).unwrap();
    let schnorrkel_key: schnorrkel::Keypair =
        mini.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);

    (local_key, schnorrkel_key)
}

pub fn load_or_create_identity() -> libp2p::identity::Keypair {
    let entry = Entry::new("my_app", "libp2p_identity").unwrap();

    if let Ok(password_str) = entry.get_password() {
        match hex::decode(password_str) {
            Ok(bytes) => {
                let sk = libp2p::identity::ed25519::SecretKey::try_from_bytes( bytes)
                .expect("invalid key");
            let kp = identity::ed25519::Keypair::from(sk);
            return identity::Keypair::from(kp);
            }
            Err(e) => {
                log::info!("Error: {}", e);
            }
        }
    }

    let ed = identity::ed25519::Keypair::generate();
    let secret_key = hex::encode(ed.secret().as_ref());

    entry
        .set_password(&secret_key)
        .unwrap();

    return identity::Keypair::from(ed);
}

