// use std::{ops::Deref, path};
use std::sync::Mutex;

use actix_web::{App, HttpServer, middleware::Logger, web::{Data}};
// use actix_web::http::{header::ContentType, StatusCode};
use env_logger;

#[path = "../messages.rs"]
mod messages;
use messages::{Message, get_messages, master_post_message};


#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let msg_vec: Data<Mutex<Vec<Message>>> = Data::new(Mutex::new(vec![]));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(master_post_message)
            .service(get_messages)
            .app_data(Data::clone(&msg_vec))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}