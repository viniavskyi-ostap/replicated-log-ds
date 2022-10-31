use std::ops::Deref;
use std::sync::Mutex;
use std::time::Duration;

use actix::clock::sleep as async_sleep;
use actix_web::{App, get, HttpResponse, HttpServer, middleware::Logger, post, web::{self, Data}};
use actix_web::http::header::ContentType;
use env_logger;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Message {
    msg: String,
}

#[post("/private/message/")]
async fn post_message(data: Data<Mutex<Vec<Message>>>, req: web::Json<Message>) -> HttpResponse {
    let msg = req.into_inner();
    if let Ok(mut v) = data.lock() {
        async_sleep(Duration::from_millis(5000)).await;

        info!("Received message: {:?}", msg);
        v.push(msg);

        HttpResponse::Ok().body("")
    } else {  // poisoned mutex
        HttpResponse::InternalServerError().body("")
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
        .bind(("127.0.0.1", 8081))?
        .run()
        .await
}