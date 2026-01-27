use std::error::Error;
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

use crate::config::{Config, IpVersion};
use crate::{
    message::message::{GreetRequest, GreetResponse},
    p2p::{
        behaviour::{Behaviour as AgentBehaviour},
    },
};

pub struct BootstrapNode {
    pub addr: Multiaddr,
    pub peer_id: PeerId,
}

pub struct P2P {
    local_key: identity::Keypair,
    cfg: Config,
    bootstrap_node_addr: Option<BootstrapNode>,
}

impl P2P {
      pub fn new(local_key: identity::Keypair, cfg: Config) -> Result<Self, Box<dyn Error>> {

        let bootstrap = match (
            &cfg.network.bootstrap_ip,
            cfg.network.bootstrap_port,
            &cfg.network.bootstrap_peer_id,
        ) {
            (Some(ip), Some(port), Some(peer_id_str)) => {
                let peer_id = peer_id_str.parse::<PeerId>()?;

                let addr_str = match cfg.network.ip_version {
                    IpVersion::Ipv4 => {
                        format!("/ip4/{}/tcp/{}/ws/p2p/{}", ip, port, peer_id)
                    }
                    IpVersion::Ipv6 => {
                        format!("/ip6/{}/tcp/{}/ws/p2p/{}", ip, port, peer_id)
                    }
                };

                let addr = addr_str.parse::<Multiaddr>()?;
                Some(BootstrapNode { addr, peer_id })
            }
            _ => None,
        };

        Ok(Self {
            local_key,
            cfg,
            bootstrap_node_addr: bootstrap,
        })
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

            let listen_addr = match self.cfg.network.ip_version {
                IpVersion::Ipv4 => format!(
                    "/ip4/{}/tcp/{}",
                    self.cfg.network.listen_ip,
                    self.cfg.network.listen_port
                ),
                IpVersion::Ipv6 => format!(
                    "/ip6/{}/tcp/{}",
                    self.cfg.network.listen_ip,
                    self.cfg.network.listen_port
                ),
            };

            swarm.listen_on(listen_addr.parse()?)?;


        if let Some(bootstrap) = &self.bootstrap_node_addr {
            swarm.dial(bootstrap.addr.clone())?;
        }


        Ok(swarm)
    }
}
