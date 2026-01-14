use libp2p::PeerId;
use std::collections::{HashMap, HashSet};

pub struct Agent {
    pub node_list: HashMap<PeerId, String>,
}

impl Agent {
    pub fn new() -> Self {
        Self {
            node_list: HashMap::new(),
        }
    }
}
