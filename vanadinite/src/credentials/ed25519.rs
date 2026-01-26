use libp2p::identity;


pub fn generate_ed25519_key_id() -> (libp2p::identity::Keypair){
    let bytes = [250, 117, 135, 27, 36, 145, 93, 153, 158, 82, 26, 83, 0, 230, 57, 134, 169, 27, 211, 5, 36, 233, 32, 222, 140, 229, 119, 87, 255, 83, 217, 89];
    let sk = libp2p::identity::ed25519::SecretKey::try_from_bytes(bytes).expect("invalid key");
    let kp = identity::ed25519::Keypair::from(sk);
    let local_key = identity::Keypair::from(kp);

    (local_key)
}