use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use actix_web::http::header::ContentType;
use actix_web::{get, web::Data, HttpResponse};
use log::info;
use serde_json::{self, Error as SerdeError};

use crate::rlog::structs::{Message, MessageID, MessageResponse};

pub type MessagesAppData = Data<RwLock<BTreeMap<MessageID, Arc<Message>>>>;

pub fn append_message(data: MessagesAppData, msg_ptr: Arc<Message>, id: MessageID) {
    let mut v = data.write().unwrap();
    info!("Appending message: {:?}", msg_ptr);
    v.insert(id, Arc::clone(&msg_ptr));
}

#[get("/public/messages/")]
pub async fn get_messages(data: MessagesAppData) -> Result<HttpResponse, SerdeError> {
    let messages: Vec<MessageResponse>;
    {
        let messages_map = data.read().unwrap();
        messages = messages_map
            .iter()
            .map(|(k, v)| MessageResponse {
                id: k.0,
                msg_ptr: Arc::clone(v),
            })
            .collect();
    }

    let response_json = serde_json::to_string(&messages)?;
    let response = HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_json);
    Ok(response)
}
