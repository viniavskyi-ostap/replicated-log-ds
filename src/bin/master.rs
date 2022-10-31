use std::ops::Deref;
use std::sync::Mutex;

use actix_web::{App, get, HttpResponse, HttpServer, middleware::Logger, post, web::{self, Data}};
use actix_web::http::{header::ContentType, StatusCode};
use env_logger;
use futures::future::join_all;
use log::info;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

const SECONDARY_URLS: [&str; 2] = ["127.0.0.1:8081", "127.0.0.1:8082"];
const SECONDARY_PATH: &str = "private/message";

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Message {
    msg: String,
}

#[post("/public/message/")]
async fn post_message(data: Data<Mutex<Vec<Message>>>, req: web::Json<Message>) -> HttpResponse {
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

#[get("/public/messages/")]
async fn get_messages(data: Data<Mutex<Vec<Message>>>) -> HttpResponse {
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

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let msg_vec: Data<Mutex<Vec<Message>>> = Data::new(Mutex::new(vec![]));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(post_message)
            .service(get_messages)
            .app_data(Data::clone(&msg_vec))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}