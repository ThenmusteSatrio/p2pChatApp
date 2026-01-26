use std::time::Duration;

use libp2p::kad;
use libp2p::{
    core::{
        transport::{upgrade, OrTransport},
        Transport,
    },
    identify, identity, noise, request_response,
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm, SwarmBuilder,
};

use crate::{
    message::message::{GreetRequest, GreetResponse},
    p2p::{
        behaviour::{Behaviour as AgentBehaviour},
    },
};

pub struct P2P {
    local_key: identity::Keypair,
    bootstrap_node_addr: Option<String>,
}

impl P2P {
    pub fn new(local_key: identity::Keypair) -> Self {
        Self {
            local_key: local_key,
            bootstrap_node_addr: Some(
                "12D3KooWJ5VBBryqyPrBXAd28fk9KsH3pXdiXshH6gpsLWWi6WiH".to_string(),
            ),
        }
    }

    pub async fn create_p2p(&mut self) -> Result<Swarm<AgentBehaviour>, Box<dyn std::error::Error>> {
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
