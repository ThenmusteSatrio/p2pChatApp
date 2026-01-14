use std::collections::{HashMap};
use libp2p::{kad::QueryId};




pub struct Agent {
    pub pending_block_queries: HashMap<QueryId, u64>,
    pub current_block_index: u64,
    pub required_quorum: usize,
    pub confirmations: usize,
    pub block_index: u64,
    pub last_hash: String,
}

impl Agent {
    pub fn new() -> Self{
        Self{
            current_block_index: 0,
            pending_block_queries: HashMap::new(),
            required_quorum: 0,
            confirmations: 0,
            block_index: 0,
            last_hash: '0'.to_string()
        }
    }

    
}