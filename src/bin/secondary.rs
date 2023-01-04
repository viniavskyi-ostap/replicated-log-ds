use std::collections::BTreeMap;
use std::env;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use rand::Rng;
use actix::clock::sleep as async_sleep;
use actix_web::{
    App,
    HttpResponse,
    HttpServer,
    middleware::Logger, post, get, web::{self, Data},
};
use actix_web::error::InternalError;
use env_logger;
use log::info;
use rand::seq::SliceRandom;

use rlog::common::{append_message, get_messages, MessageLog};
use rlog::structs::{Message, MessageID, SecondaryMessageRequest};

mod rlog;

#[post("/private/message/")]
async fn post_message(
    data_mes_log: Data<MessageLog>,
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
    append_message(data_mes_log, request.msg_ptr, request.id.clone());

    let num = rand::thread_rng().gen_range(0..100);
    if num > 50 { return HttpResponse::InternalServerError().body("Not Success");}
    HttpResponse::Ok().body("Success")
}

#[get("/private/health/")]
async fn get_health() -> HttpResponse{
    HttpResponse::Ok().finish()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let args: Vec<String> = env::args().collect();
    let port = args[1].parse::<u16>().unwrap();
    let data_mes_log = Data::new(RwLock::new(BTreeMap::<MessageID, Arc<Message>>::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(post_message)
            .service(get_messages)
            .service(get_health)
            .app_data(Data::clone(&data_mes_log))
    })
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
