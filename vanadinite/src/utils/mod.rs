use libp2p::PeerId;
use sha2::{Digest, Sha256};

// fungsi untuk membuat hash dari keseluruhan peer id yang berada di jaringan
pub fn hash_peer_list(peers: &[PeerId]) -> [u8; 32] {
    let mut sorted = peers.to_vec();
    sorted.sort();
    let mut hasher = Sha256::new();
    for peer in sorted {
        hasher.update(peer.to_bytes());
    }
    hasher.finalize().into()
}

pub fn get_session_id() -> u32{
    return 1;
}