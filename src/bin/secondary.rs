use std::collections::BTreeMap;
use std::env;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use actix::clock::sleep as async_sleep;
use actix_web::{
    App,
    get,
    HttpResponse,
    HttpServer, middleware::Logger, post, web::{self, Data},
};
use env_logger;
use log::info;
use rand::Rng;
use rand::seq::SliceRandom;

use rlog::common::{append_message, get_messages, MessageLogAppData};
use rlog::config::SECONDARY_SLEEP_DURATIONS;
use rlog::structs::{Message, MessageID, SecondaryMessageRequest};

mod rlog;

#[post("/private/message/")]
async fn post_message(
    data_mes_log: MessageLogAppData,
    request: web::Json<SecondaryMessageRequest>,
) -> HttpResponse {
    let request = request.into_inner();
    let sleep_dur = *SECONDARY_SLEEP_DURATIONS
        .choose(&mut rand::thread_rng())
        .unwrap();
    info!("Sleeping for: {:?}", sleep_dur);
    async_sleep(Duration::from_secs(sleep_dur)).await;
    append_message(data_mes_log, request.msg_ptr, request.id.clone());

    let num = rand::thread_rng().gen_range(0..100);
    if num > 50 { return HttpResponse::InternalServerError().body("Not Success"); }
    HttpResponse::Ok().body("Success")
}

#[get("/private/health/")]
async fn get_health() -> HttpResponse {
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
