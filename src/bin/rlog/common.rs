use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

use actix_web::{get, HttpResponse, web::Data};
use actix_web::http::header::ContentType;
use log::info;
use serde_json;

use crate::rlog::structs::{Message, MessageID, MessageResponse};

pub type MessageLogAppData = Data<RwLock<BTreeMap<MessageID, Arc<Message>>>>;


pub fn append_message(data: MessageLogAppData, msg_ptr: Arc<Message>, id: MessageID) {
    let mut v = data.write().unwrap();
    info!("Appending message: {:?}", msg_ptr);
    v.insert(id, Arc::clone(&msg_ptr));
}


#[get("/public/messages/")]
pub async fn get_messages(data: MessageLogAppData) -> Result<HttpResponse, Box<dyn Error>> {
    let mut messages: Vec<MessageResponse> = vec![];
    {
        let messages_map = data.read().unwrap();
        // Check if messages id`s are sequential
        for (idx, (k, v)) in messages_map.iter().enumerate() {
            if k.0 != idx { break; }
            messages.push(MessageResponse { id: k.0, msg_ptr: Arc::clone(v) });
        }
    }
    let response_json = serde_json::to_string(&messages)?;
    let response = HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_json);
    Ok(response)
}
