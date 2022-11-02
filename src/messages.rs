use std::{ops::Deref};
use std::sync::Mutex;
use actix::clock::sleep as async_sleep;
use std::time::Duration;
use futures::future::join_all;

use actix_web::{get, HttpResponse, web::{self, Data}, post};
use actix_web::http::{header::ContentType, StatusCode};
// use env_logger;
// use futures::future::join_all;
use log::info;
// use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Message {
    msg: String,
}

const SECONDARY_URLS: [&str; 2] = ["secondary:8081", "secondary2:8082"];
const SECONDARY_PATH: &str = "private/message";

#[get("/public/messages/")]
pub async fn get_messages(data: Data<Mutex<Vec<Message>>>) -> HttpResponse {
    if let Ok(v) = data.lock() {
        if let Ok(vec_json) = serde_json::to_string(v.deref()) {
            info!("All messages sent!");

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(vec_json)
        } else {  // cannot serialize
            HttpResponse::InternalServerError().body("")
        }
    } else {  // poisoned mutex
        HttpResponse::InternalServerError().body("")
    }
}


#[post("/private/message/")]
pub(crate) async fn slave_post_message(data: Data<Mutex<Vec<Message>>>, req: web::Json<Message>) -> HttpResponse {
    let msg = req.into_inner();
    async_sleep(Duration::from_millis(10000)).await;
    if let Ok(mut v) = data.lock() {
        info!("Secondary received message: {:?}", msg);
        v.push(msg);

        HttpResponse::Ok().body("")
    } else {  // poisoned mutex
        HttpResponse::InternalServerError().body("")
    }
}


#[post("/public/message/")]
pub async fn master_post_message(data: Data<Mutex<Vec<Message>>>, req: web::Json<Message>) -> HttpResponse {
    let msg = req.into_inner();

    let client = reqwest::Client::new();
    let responses = SECONDARY_URLS.map(|address| {
        let url = format!("http://{}/{}/", address, SECONDARY_PATH);
        client.post(url).json(&msg).send()
    });
    let responses = join_all(responses).await;

    for response in responses {
        if let Ok(response) = response {
            if response.status() != StatusCode::OK {
                return HttpResponse::InternalServerError().body("")
            }
        } else {
            return HttpResponse::InternalServerError().body("")
        }
    }

    if let Ok(mut v) = data.lock() {
        info!("Master received message: {:?}", msg);
        v.push(msg);
        HttpResponse::Ok().body("")
    } else {  // poisoned mutex
        return HttpResponse::InternalServerError().body("");
    }
}
