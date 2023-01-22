use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::config::SECONDARY_URLS;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Message {
    text: String,
}

#[derive(Deserialize, Debug, Serialize, Clone, Copy)]
pub struct WriteConcern(pub(crate) usize);

impl Default for WriteConcern {
    fn default() -> Self {
        WriteConcern(SECONDARY_URLS.len() + 1)
    }
}

#[derive(Deserialize, Debug, Serialize, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MessageID(pub usize);

impl Default for MessageID {
    fn default() -> Self {
        MessageID(0)
    }
}

#[derive(Deserialize, Debug, Serialize, Clone, Copy)]
pub struct QuorumAppend(pub bool);

impl Default for QuorumAppend {
    fn default() -> Self {
        QuorumAppend(false)
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct MasterMessageRequest {
    #[serde(alias = "message")]
    pub msg_ptr: Arc<Message>,
    #[serde(default)]
    pub wc: WriteConcern,
    #[serde(default)]
    pub quorum_append: QuorumAppend,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct SecondaryMessageRequest {
    #[serde(alias = "message")]
    pub msg_ptr: Arc<Message>,
    pub id: MessageID,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct MessageResponse {
    #[serde(rename = "message")]
    pub msg_ptr: Arc<Message>,
    pub id: usize,
}
