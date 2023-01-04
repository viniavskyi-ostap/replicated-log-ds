use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}, RwLock};
use std::time::Duration;

use actix_web::{
    App, HttpResponse, HttpServer,
    middleware::Logger, post, web::{self, Data}};
use actix_web::http::StatusCode;
use env_logger;
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest;
use tokio::{self, time};

use rlog::common::{
    append_message, get_messages,
    MessageLogAppData, SECONDARY_URLS};
use rlog::heartbeat::{
    get_secondaries_health, HealthStatus,
    HealthStatusListAppData, update_health_status};
use rlog::structs::{MasterMessageRequest, Message, MessageID, SecondaryMessageRequest};

mod rlog;

static GLOBAL_MESSAGES_COUNT: AtomicUsize = AtomicUsize::new(0);

#[post("/public/message/")]
async fn post_message(
    data_mes_log: MessageLogAppData,
    data_health_status: HealthStatusListAppData,
    request: web::Json<MasterMessageRequest>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let request = request.into_inner();
    let write_concern = request.wc.0;

    // Check quorum before appending messages!
    if request.quorum_append.0 && !check_quorum(write_concern, data_health_status.clone()) {
        return Ok(HttpResponse::InternalServerError().body("No quorum for request!"));
    }

    let message_id = MessageID(GLOBAL_MESSAGES_COUNT.fetch_add(1, Ordering::AcqRel));

    // Send messages with retry logic to secondary!
    let req_futures = FuturesUnordered::new();
    SECONDARY_URLS.iter().enumerate().for_each(|(sec_id, address)| {
        let msg_ptr = Arc::clone(&request.msg_ptr);
        let data_health_status = data_health_status.clone();

        let future = tokio::spawn(async move {
            let url = format!("http://{}/{}/", address, "private/message");
            let secondary_request = SecondaryMessageRequest { msg_ptr, id: message_id };
            send_message(secondary_request, url, sec_id, data_health_status).await
        });
        req_futures.push(future);
    });

    // Append message on master
    append_message(data_mes_log, request.msg_ptr, message_id);
    let take_n = write_concern - 1;
    let _ = req_futures.take(take_n).collect::<Vec<_>>().await;
    Ok(HttpResponse::Ok().body("Success"))
}

fn check_quorum(write_concern: usize, data_health_status: HealthStatusListAppData) -> bool {
    let health = data_health_status.read().unwrap();
    let alive_n = health.iter().filter(|&n| *n == HealthStatus::ALIVE).count();
    return alive_n >= (write_concern - 1);
}


async fn send_message(msg_req: SecondaryMessageRequest, url: String, sec_id: usize,
                      data_health_status: HealthStatusListAppData) {
    let mut health_status: HealthStatus;
    let client = reqwest::Client::new();
    let request = client
        .post(url)
        .timeout(Duration::from_secs(15))
        .json(&msg_req);

    let mut retry_interval = time::interval(Duration::from_secs(5));
    loop {
        { health_status = data_health_status.read().unwrap()[sec_id]; }
        if health_status == HealthStatus::DEAD {
            retry_interval.tick().await;
            continue;
        }

        match request.try_clone().unwrap().send().await {
            Ok(response) if response.status() == StatusCode::OK => break,
            _ => continue
        }
    }
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let messages_log = Data::new(RwLock::new(BTreeMap::<MessageID, Arc<Message>>::new()));
    let secondaries_health = Data::new(RwLock::new([HealthStatus::DEAD; SECONDARY_URLS.len()]));
    let secondaries_health_clone = secondaries_health.clone();

    // start heartbeat checking background task
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            update_health_status(secondaries_health_clone.clone()).await;
            interval.tick().await;
        }
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(post_message)
            .service(get_messages)
            .service(get_secondaries_health)
            .app_data(Data::clone(&messages_log))
            .app_data(Data::clone(&secondaries_health))
    })
        .bind(("0.0.0.0", 8080))?
        .run().await
}