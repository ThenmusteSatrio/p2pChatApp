use libp2p::{PeerId, Swarm};

use crate::{ ChatMessage, message::message::GreetRequest, p2p::behaviour::Behaviour as AgentBehaviour};


pub enum P2PCommand {
    SendGreet { peer: PeerId, msg: String },
    SendChat { peer: PeerId, msg: ChatMessage },
    FindNode { peer: PeerId },
}

pub fn handle_command(cmd: P2PCommand, swarm: &mut Swarm<AgentBehaviour>) {
    match cmd {
        P2PCommand::SendGreet { peer, msg } => {
            swarm
                .behaviour_mut()
                .rr
                .send_request(&peer, GreetRequest::Syn { message: msg });
        },
        P2PCommand::SendChat { peer, msg } => {
            log::info!("send chat: {msg:?}");
            swarm
                .behaviour_mut()
                .rr
                .send_request(&peer, GreetRequest::Chat{ message: msg });
        },
        P2PCommand::FindNode { peer } => {
            swarm
                .behaviour_mut()
                .kad
                .get_closest_peers(peer);
        }
    }
}