use libp2p::{PeerId, Swarm, identify, kad, request_response, swarm::SwarmEvent};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::{
    ChatMessage, PeerStore, StoredPeer, message::message::{GreetRequest, GreetResponse}, p2p::behaviour::{Behaviour as AgentBehaviour, Event as AgentEvent}
};

pub enum P2PEvent {
    PeerDiscovered(PeerId),
    MessageReceived { peer: PeerId, msg: ChatMessage },
}

pub async fn handle_swarm_event(event: SwarmEvent<AgentEvent>, swarm: &mut Swarm<AgentBehaviour>, peer_store: &PeerStore, event_tx: &mpsc::Sender<P2PEvent>) {
    match event {
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
            SwarmEvent::Behaviour(AgentEvent::Identify(event)) => match event {
                identify::Event::Sent { connection_id, peer_id } => info!("Sent: {connection_id} | {peer_id}"),
                identify::Event::Pushed { connection_id, peer_id, info } => info!("Pushed: {connection_id} | {peer_id} | {info:?}"),
                identify::Event::Received { connection_id: _, peer_id, info } => {
                    let mut peers = peer_store.peers.write().await;

                    let entry = peers.entry(peer_id).or_insert(StoredPeer { peer_id: peer_id.to_string(), addrs: vec![], last_seen: chrono::Utc::now().timestamp(), success: 0, fail: 0 });
                    entry.addrs = info.listen_addrs.iter().map(|a| a.to_string()).collect();

                    entry.last_seen = chrono::Utc::now().timestamp();
                },
                _ => {}
            }
            SwarmEvent::Behaviour(AgentEvent::RequestResponse(event)) => match event {
                request_response::Event::Message { peer, connection_id:_, message } => {
                    match message {
                        request_response::Message::Request { request_id, request, channel } => {
                            info!("request_response::Event::Message::Request -> PeerID: {peer} | RequestID: {request_id} | RequestMessage: {request:?}");
                            match request {
                                GreetRequest::Syn { message } => {}
                                GreetRequest::Chat { message } => {
                                    let _ = event_tx.send(P2PEvent::MessageReceived { peer, msg: message }).await;
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
            //@ Event dipicu ketika ada query
            SwarmEvent::Behaviour(AgentEvent::Kad(kad::Event::OutboundQueryProgressed { id, result, stats: _, step:_ })) => {
                if let kad::QueryResult::GetClosestPeers(Ok(ok)) = result {
                    
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
                    let mut peers = peer_store.peers.write().await;

                    let entry = peers.entry(peer).or_insert(StoredPeer {
                        peer_id: peer.to_string(),
                        addrs: vec![],
                        last_seen: chrono::Utc::now().timestamp(),
                        success: 0,
                        fail: 0,
                    });

                    entry.addrs = addresses.iter().map(|a| a.to_string()).collect();
                    entry.last_seen = chrono::Utc::now().timestamp();
                }
                kad::Event::OutboundQueryProgressed { id, result, stats, step } => {
                    info!("kad::Event::OutboundQueryProgressed: ID: {id:?} | Result: {result:?} | Stats: {stats:?} | Step: {step:?}");
                }
                _ => {}
            }
            _ => {}
    }
}