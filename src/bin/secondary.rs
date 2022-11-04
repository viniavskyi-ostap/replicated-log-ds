use std::sync::Mutex;

use actix_web::{App, HttpServer, middleware::Logger, web::{Data}};
use env_logger;
use std::env;

#[path = "../messages.rs"]
mod messages;
use messages::{Message, get_messages, slave_post_message, };


#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let args: Vec<String> = env::args().collect();
    let port = args[1].parse::<u16>().unwrap();

    let msg_vec: Data<Mutex<Vec<Message>>> = Data::new(Mutex::new(vec![]));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(slave_post_message)
            .service(get_messages)
            .app_data(Data::clone(&msg_vec))
    })
        .bind(("0.0.0.0", port))?
        .run()
        .await
}