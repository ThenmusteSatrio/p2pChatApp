use serde::{Deserialize, Serialize};

use crate::ChatMessage;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GreetRequest {
    Syn { message: String },
    Chat{ message: ChatMessage },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GreetResponse {
    Ack { message: String },
}
