pub mod credentials;
mod message;
mod p2p;
mod utils;

use std::{error::Error};

use libp2p::kad;

use crate::p2p::p2p::P2P;

#[tokio::main]
// #[warn(unused_imports)]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    let local_key = credentials::ed25519::generate_ed25519_key_id();

    let mut p2p_connection_event = P2P::new(local_key);
    match p2p_connection_event.create_p2p().await {
        Ok(mut swarm) => {
            p2p_connection_event
                .event_handler(&mut swarm)
                .await;
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    Ok(())
}
