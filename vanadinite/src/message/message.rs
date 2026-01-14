use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GreetRequest {
    Syn{ message: String},
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GreetResponse {
    Ack{message: String}
}