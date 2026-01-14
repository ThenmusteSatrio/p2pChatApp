use crate::message::message::{GreetRequest, GreetResponse};
use crate::p2p::behaviour::{Behaviour as AgentBehaviour, Event as AgentEvent};
use libp2p::core::transport::upgrade;
use libp2p::identity::Keypair;
use libp2p::{identify,  noise, tcp, yamux, PeerId, StreamProtocol, Swarm, SwarmBuilder, swarm::SwarmEvent, core::transport::OrTransport, core::Transport};
use libp2p::{request_response};
use std::{error::Error, time::Duration};

use libp2p::futures::StreamExt;
use libp2p::kad::{self, RoutingUpdate};
use tracing::{error, info, warn};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum NodeState{
    Default,
    Leader,
    // Follower
    Receiver,
    Sender
}

 #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Role {
    Client,
    Server,
}

pub struct P2P {
    local_key: Keypair,
}

impl P2P {
    pub fn new(local_key: Keypair) -> Self {
        Self {
            local_key: local_key
        }
    }


    pub async fn create_p2p(&mut self) -> Result<Swarm<AgentBehaviour>, Box<dyn Error>> {
        let local_key = self.local_key.clone();
        
        let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_other_transport(move |keypair| {
            let tcp_transport:  libp2p::tcp::Transport<libp2p::tcp::tokio::Tcp> = tcp::Transport::new(tcp::Config::default().nodelay(true));
            let ws_transport = libp2p::websocket::Config::new(tcp::tokio::Transport::new(tcp::Config::default()));

            OrTransport::new(ws_transport, tcp_transport)
                .upgrade(upgrade::Version::V1)
                .authenticate(noise::Config::new(&local_key).unwrap())
                .multiplex(yamux::Config::default())
                .boxed()
        })?
        .with_behaviour(|keypair| {
            let local_peer_id = PeerId::from(keypair.clone().public());

            let kad_config = kad::Config::default();
            let kad_memory = kad::store::MemoryStore::new(local_peer_id);
            let kad = kad::Behaviour::with_config(local_peer_id, kad_memory, kad_config);

            let identity_config = identify::Config::new(
                "/agent/connection/1.0.0".to_string(),
                keypair.clone().public(),
            )
            .with_push_listen_addr_updates(true)
            .with_interval(Duration::from_secs(30));
            let identify = identify::Behaviour::new(identity_config);

            let rr_config = request_response::Config::default();
            let rr_protocol = StreamProtocol::new("/agent/message/1.0.0");
            let rr_behavior =
                request_response::cbor::Behaviour::<GreetRequest, GreetResponse>::new(
                    [(rr_protocol, request_response::ProtocolSupport::Full)],
                    rr_config,
                );

            AgentBehaviour::new(kad, identify, rr_behavior)
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(30)))
        .build();

        swarm.behaviour_mut().set_server_mode();
        swarm.listen_on("/ip4/0.0.0.0/tcp/8000/ws".parse()?)?;
        

        Ok(swarm)
    }
    
    pub async fn event_handler(&mut self, swarm: &mut Swarm<AgentBehaviour>) {
        loop {
                match swarm.select_next_some().await {
                    SwarmEvent::NewListenAddr {
                        listener_id,
                        address,
                    } => info!("NewListenAddr: {listener_id:?} | {address:?}"),
                    SwarmEvent::ConnectionEstablished {
                        peer_id,
                        connection_id,
                        endpoint,
                        num_established,
                        concurrent_dial_errors,
                        established_in,
                    } => info!("ConnectionEstablished: {peer_id:?} | {connection_id:?} | {endpoint:?} | {num_established:?} | {concurrent_dial_errors:?} | {established_in:?}"),
                    SwarmEvent::Dialing { peer_id, connection_id } => info!("Dialing: {peer_id:?} | {connection_id}"),

                    //@ Prosess identifikasi local node ke jaringan / peer
                    SwarmEvent::Behaviour(AgentEvent::Identify(event)) => match event {
                        identify::Event::Sent { connection_id, peer_id } => info!("Sent: {connection_id} | {peer_id}"),
                        identify::Event::Pushed { connection_id, peer_id, info } => info!("Pushed: {connection_id} | {peer_id} | {info:?}"),
                        //^ menerima informasi identitas (peer lain) dari jaringan
                        identify::Event::Received { connection_id: _, peer_id, info } => {
                            info!("IdentifyEvent:Received: {peer_id} | {info:?}");
                            
                            for addr in info.clone().listen_addrs{
                                let agent_routing = swarm.behaviour_mut().kad.add_address(&peer_id, addr.clone());
                                match agent_routing {
                                    RoutingUpdate::Failed => error!("IdentifyReceived: Failed to register address to Kademlia"),
                                    RoutingUpdate::Pending => warn!("IdentifyReceived: Register address pending"),
                                    RoutingUpdate::Success => {
                                        info!("IdentifyReceived: {addr}: Success register address");
                                    } 
                                }

                                _= swarm.behaviour_mut().register_addr_rr(&peer_id, addr.clone());
                            }
                           
                        },

                        identify::Event::Error { connection_id, peer_id, error } => {}
                    }

                    SwarmEvent::Behaviour(AgentEvent::RequestResponse(event)) => match event {
                        request_response::Event::Message { peer, connection_id:_, message } => {
                            match message {
                                request_response::Message::Request { request_id, request, channel } => {
                                    info!("request_response::Event::Message::Request -> PeerID: {peer} | RequestID: {request_id} | RequestMessage: {request:?}");
                                    match request {
                                         GreetRequest::Syn { message } => {
                                            info!("Message: {message}");
                                         }
                                    }
                                }
                                
                                request_response::Message::Response { request_id, response } => {
                                    info!(" request_response::Event::Message::Response -> PeerID: {peer} | RequestID: {request_id} | ResponseMessage: {response:?}");
                                    match response {
                                         GreetResponse::Ack { message } => {}
                                    }
                                }
                            }
                        }
                        request_response::Event::InboundFailure { peer, connection_id, request_id, error } => {
                            warn!("request_response::Event::InboundFailure -> PeerID: {peer} | ConnectionID: {connection_id} | RequestID: {request_id} | Error: {error:?}")
                        }
                        request_response::Event::OutboundFailure { peer, connection_id, request_id, error } => {
                            warn!("request_response::Event::OutboundFailure -> PeerID: {peer} | ConnectionID: {connection_id} | RequestID: {request_id} | Error: {error:?}")
                        }
                        request_response::Event::ResponseSent { peer, connection_id, request_id } => {
                            info!("request_response::Event::ResponseSent -> PeerID: {peer} | ConnectionID: {connection_id} | RequestID: {request_id}")
                        }
                    }

                    SwarmEvent::Behaviour(AgentEvent::Kad(kad::Event::OutboundQueryProgressed { id, result, stats: _, step:_ })) => {
                        if let kad::QueryResult::GetClosestPeers(Ok(ok)) = result {
                            info!("Query {} selesai. Ditemukan {} peer", id, ok.peers.len());
                        }
                    }

                
                    SwarmEvent::Behaviour(AgentEvent::Kad(event)) => match event {
                        kad::Event::ModeChanged { new_mode }=> {
                            info!("ModeChanged: {new_mode:?}")
                        }
                        kad::Event::RoutablePeer { peer, address } => {
                            info!("kad::Event::RoutablePeer -> PeerID: {peer} | Address: {address}")
                        }
                        kad::Event::PendingRoutablePeer { peer, address } => {
                            info!("kad::Event::PendingRoutablePeer -> PeerID: {peer} | Address: {address}")
                        }
                        kad::Event::InboundRequest { request } => {
                            info!("kad::Event::InboundRequest -> Request: {request:?}")
                        }
                        kad::Event::RoutingUpdated { peer, is_new_peer, addresses, bucket_range, old_peer } => {
                            info!("kad::Event::RoutingUpdated: {peer} | IsNewPeer: {is_new_peer} | Addresses: {addresses:?} | BucketRange: {bucket_range:?} | OldPeer: {old_peer:?}");
                        }
                        kad::Event::OutboundQueryProgressed { id, result, stats, step } => {
                            info!("kad::Event::OutboundQueryProgressed: ID: {id:?} | Result: {result:?} | Stats: {stats:?} | Step: {step:?}");
                        }
                        _ => {}
                    }
                    _ => {}
                }
                
            }

    }

}
