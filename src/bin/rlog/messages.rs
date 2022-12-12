use std::collections::HashMap;
use std::{ops::Deref};
use std::sync::{Mutex, PoisonError, MutexGuard};
use itertools::Itertools;
use actix::fut::ok;
use actix_web::{get, HttpResponse, web::Data};
use actix_web::http::{header::ContentType};
use log::info;
use serde::de::value::Error;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io;
#[derive(Deserialize, Serialize, Clone, Debug )]
pub struct Message {
    msg: String,
    
    #[serde(default)]
    w_concern: WriteConcern,

    #[serde(default)]
    id: MessageID,
}

#[derive(Deserialize, Debug, Serialize, Clone, Copy)]
pub struct WriteConcern(usize);

#[derive(Deserialize, Debug, Serialize, Clone, Copy, PartialEq, Eq, Hash,  PartialOrd, Ord)]
pub struct MessageID(usize);

impl Message{
    pub fn concern(&self) -> usize { self.w_concern.0}
    pub fn get_id(&self) -> MessageID { self.id}
    pub fn set_id(&mut self, id: usize) -> () {self.id = MessageID(id)}
}

impl Default for WriteConcern{
    fn default() -> Self {
        WriteConcern(3)
    }
}
impl Default for MessageID{
    fn default() -> Self {
        MessageID(0)
    }
}



pub fn save_message(data: Data<Mutex<HashMap<MessageID, Message>>>,  msg: Message) -> Result<(), String>
{
    let mut v = data.lock()
    .map_err(
        |e| e.to_string()
    )?;
    info!("Master received message: {:?}", msg);
    // v.push(msg);
    v.insert(msg.get_id(), msg);
    return Ok(());
}

fn send_messages(message_map:MutexGuard< HashMap<MessageID, Message> >) -> HttpResponse
{
    let mut msgs: Vec<(usize, String)> = vec![];
    for key in message_map.keys().sorted() {
        msgs.push((key.0, message_map[key].msg.clone()));
    }
    match serde_json::to_string(msgs.deref()).map_err(|err|->String{err.to_string()}) {
        Ok(s) => HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(s),
        
        Err(s) => HttpResponse::InternalServerError().body(s)
    }
}

#[get("/public/messages/")]
pub async fn get_messages(data: Data<Mutex<HashMap<MessageID, Message>>>) -> HttpResponse {
    
    match data.lock().map_err(|err| -> String {err.to_string()}){
        Ok(msgs) => send_messages(msgs),
        Err(s)=> HttpResponse::InternalServerError().body(s) 
    }

}



