use std::collections::BTreeMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use actix::clock::sleep as async_sleep;
use actix_web::{
    middleware::Logger,
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use env_logger;
use log::info;
use rand::seq::SliceRandom;

use rlog::common::{append_message, get_messages, MessagesAppData};
use rlog::structs::{Message, MessageID, SecondaryMessageRequest};

mod rlog;

#[post("/private/message/")]
async fn post_message(
    data: MessagesAppData,
    request: web::Json<SecondaryMessageRequest>,
) -> HttpResponse {
    let request = request.into_inner();
    let sleeps: Vec<u64> = vec![1000, 5000, 10000];
    let sleep_dur = sleeps
        .choose(&mut rand::thread_rng())
        .unwrap_or(&10u64)
        .clone();
    info!("Sleeping for: {:?}", sleep_dur);

    async_sleep(Duration::from_millis(sleep_dur)).await;

    append_message(data, request.msg_ptr, request.id);
    HttpResponse::Ok().body("Success")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args: Vec<String> = env::args().collect();
    let port = args[1].parse::<u16>().unwrap();

    let app_data = Data::new(Mutex::new(BTreeMap::<MessageID, Arc<Message>>::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(post_message)
            .service(get_messages)
            .app_data(Data::clone(&app_data))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
