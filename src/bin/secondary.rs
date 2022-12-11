use std::{sync::Mutex, collections::HashMap, hash::Hash};

use actix_web::{App, HttpServer, HttpResponse, middleware::Logger, web::{self, Data}, post};
use env_logger;
use std::env;
use std::time::Duration;
use actix::clock::sleep as async_sleep;

mod rlog;
use rlog::messages::{Message, get_messages, save_message, MessageID };
use rand::seq::SliceRandom;
use log::info;



#[post("/private/message/")]
async fn post_message(data: Data<Mutex<HashMap<MessageID, Message>>>, req: web::Json<Message>) -> HttpResponse {
    let msg = req.into_inner();
    let sleeps:Vec<u64> = vec![1000,5000, 10000];
    let sleep_dur = sleeps.choose(&mut rand::thread_rng()).unwrap_or(&10u64).clone();
    info!("Sleeping for: {:?}", sleep_dur);
    
    async_sleep(Duration::from_millis(sleep_dur)).await;
    
    if let Err(_) = save_message(data, msg) { return  HttpResponse::InternalServerError().body("body"); }
    HttpResponse::Ok().body("")
}



#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let args: Vec<String> = env::args().collect();
    let port = args[1].parse::<u16>().unwrap();
    let map : HashMap<MessageID, Message> = HashMap::new();
    let msg_vec: Data<Mutex<HashMap<MessageID, Message>>> = Data::new(Mutex::new(map));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(post_message)
            .service(get_messages)
            .app_data(Data::clone(&msg_vec))
    })
        .bind(("0.0.0.0", port))?
        .run()
        .await
}