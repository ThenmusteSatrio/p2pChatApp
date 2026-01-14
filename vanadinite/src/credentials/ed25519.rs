use libp2p::identity;
use schnorrkel::MiniSecretKey;


pub fn generate_ed25519_key_id() -> (libp2p::identity::Keypair, schnorrkel::Keypair){
    let ed_key = identity::ed25519::Keypair::generate();
    let local_key: identity::Keypair = identity::Keypair::from(ed_key.clone());

    let secret = ed_key.secret();

    let mini = MiniSecretKey::from_bytes(secret.as_ref()).unwrap();
    let schnorrkel_key: schnorrkel::Keypair =
        mini.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);

    (local_key, schnorrkel_key)
}