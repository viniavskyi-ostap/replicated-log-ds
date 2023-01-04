use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use actix_web::http::header::ContentType;
use actix_web::{get, web::Data, HttpResponse};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{self, Error as SerdeError};

use crate::rlog::structs::{Message, MessageID, MessageResponse};

// pub type MessagesAppData = Data<MasterAppData>;
pub type MessageLog = RwLock<BTreeMap<MessageID, Arc<Message>>>;
pub type HealthStatusList = RwLock<[HealthStatus; SECONDARY_URLS.len()]>;

pub const SECONDARY_URLS: [&'static str; 2] = ["localhost:8081", "localhost:8082"];
pub const SECONDARY_PATH: &'static str = "private/message";

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum HealthStatus {
    ALIVE, DEAD
}

pub fn append_message(data: Data<MessageLog>, msg_ptr: Arc<Message>, id: MessageID){
    let mut v = data.write().unwrap();
    info!("Appending message: {:?}", msg_ptr);
    v.insert(id, Arc::clone(&msg_ptr));
}

#[get("/public/messages/")]
pub async fn get_messages(data: Data<MessageLog>) -> Result<HttpResponse, SerdeError> {
    let mut messages: Vec<MessageResponse> = vec![];
    {
        let messages_map = data.read().unwrap();
        // Check if messages id`s are sequential
        for (idx, (k, v)) in messages_map.iter().enumerate(){
            if k.0 != idx { break; }
            messages.push(MessageResponse{id: k.0, msg_ptr: Arc::clone(v)});
        }
    }
    let response_json = serde_json::to_string(&messages)?;
    let response = HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_json);
    Ok(response)
}
