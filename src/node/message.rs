use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum MessageType {
    #[serde(rename = "init")]
    Init,
    #[serde(rename = "init_ok")]
    InitOk,
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "add_ok")]
    AddOk,
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "read_ok")]
    ReadOk,
    #[serde(rename = "gossip")]
    Gossip,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: MessageBody,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MessageBody {
    #[serde(rename = "type")]
    pub typ: MessageType,

    #[serde(flatten)]
    pub message_params: HashMap<String, Value>,
}

