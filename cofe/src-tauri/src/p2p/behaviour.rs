use crate::message::message::{GreetRequest, GreetResponse};
use libp2p::kad::RoutingUpdate;
use libp2p::request_response::OutboundRequestId;
use libp2p::swarm::NetworkBehaviour;
use libp2p::{identify, kad, request_response, Multiaddr, PeerId};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
pub(crate) struct Behaviour {
    pub identify: identify::Behaviour,
    pub kad: kad::Behaviour<kad::store::MemoryStore>,
    pub rr: request_response::cbor::Behaviour<GreetRequest, GreetResponse>,
}

impl Behaviour {
    pub fn new(
        kad: kad::Behaviour<kad::store::MemoryStore>,
        identify: identify::Behaviour,
        rr: request_response::cbor::Behaviour<GreetRequest, GreetResponse>,
    ) -> Self {
        Self {
            identify: identify,
            kad: kad,
            rr: rr,
        }
    }

    pub fn register_addr_kad(&mut self, peer_id: &PeerId, addr: Multiaddr) -> RoutingUpdate {
        self.kad.add_address(peer_id, addr)
    }

    pub fn register_addr_rr(&mut self, peer_id: &PeerId, addr: Multiaddr) -> bool {
        self.rr.add_address(peer_id, addr)
    }

    pub fn send_message(&mut self, peer_id: &PeerId, message: GreetRequest) -> OutboundRequestId {
        self.rr.send_request(peer_id, message)
    }

    pub fn send_response(
        &mut self,
        ch: request_response::ResponseChannel<GreetResponse>,
        rs: GreetResponse,
    ) -> Result<(), GreetResponse> {
        self.rr.send_response(ch, rs)
    }

    pub fn set_server_mode(&mut self) {
        self.kad.set_mode(Some(kad::Mode::Server));
    }
}

#[derive(Debug)]
pub(crate) enum Event {
    Identify(identify::Event),
    Kad(kad::Event),
    RequestResponse(request_response::Event<GreetRequest, GreetResponse>),
}

impl From<identify::Event> for Event {
    fn from(value: identify::Event) -> Self {
        Self::Identify(value)
    }
}

impl From<kad::Event> for Event {
    fn from(value: kad::Event) -> Self {
        Self::Kad(value)
    }
}

impl From<request_response::Event<GreetRequest, GreetResponse>> for Event {
    fn from(value: request_response::Event<GreetRequest, GreetResponse>) -> Self {
        Self::RequestResponse(value)
    }
}
