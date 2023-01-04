use std::collections::BTreeMap;
use std::error::Error;
use std::future::Future;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use actix_web::http::StatusCode;
use actix_web::{
    middleware::Logger,
    post,
    get,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};

use serde_json::{self, Error as SerdeError};
use actix_web::http::header::ContentType;
use env_logger;
use futures::{stream::FuturesUnordered, StreamExt};
use futures::future::join_all;
use log::info;
use reqwest;
use tokio;
use tokio::time;

use rlog::common::{append_message, get_messages, SECONDARY_URLS, SECONDARY_PATH, HealthStatus, HealthStatusList, MessageLog};
use rlog::errors::WriteConcernNotSatisfiedError;
use rlog::structs::{MasterMessageRequest, Message, MessageID, SecondaryMessageRequest};
use crate::HealthStatus::{ALIVE, DEAD};

mod rlog;

static GLOBAL_MESSAGES_COUNT: AtomicUsize = AtomicUsize::new(0);

#[post("/public/message/")]
async fn post_message(
    data_mes_log: Data<MessageLog>,
    data_health_status: Data<HealthStatusList>,
    request: web::Json<MasterMessageRequest>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let request = request.into_inner();
    let write_concern = request.wc.0;

    // Check quorum before appending messages!
    if !check_quorum(write_concern, data_health_status.clone()) {
        return Ok(HttpResponse::InternalServerError().body("No quorum for request!"));
    }

    let message_id = MessageID(GLOBAL_MESSAGES_COUNT.fetch_add(1, Ordering::AcqRel));
    let message_ptr = Arc::clone(&request.msg_ptr);

    // Send messages with retry logic to secondary!
    let mut req_futures = FuturesUnordered::new();
    let _: Vec<_> = SECONDARY_URLS.iter().enumerate().map(|(sec_id, address)| {
        let msg_ptr = Arc::clone(&message_ptr);
        let id = message_id.clone();

        let data_hs_clone = data_health_status.clone();
        let future = tokio::spawn(async move {
            let url = format!("http://{}/{}/", address, SECONDARY_PATH);
            let secondary_request = SecondaryMessageRequest { msg_ptr, id };
            send_message(secondary_request, url, sec_id, data_hs_clone.clone()).await
        });
        req_futures.push(future);
    }).collect();

    // Append message on master
    append_message(data_mes_log, request.msg_ptr, message_id.clone());
    let take_n = write_concern - 1;
    let _ = req_futures.take(take_n).collect::<Vec<_>>().await;
    Ok(HttpResponse::Ok().body("Success"))
}

fn check_quorum(write_concern: usize, data_health_status: Data<HealthStatusList>) -> bool {
    let h = data_health_status.read().unwrap();
    let alive_n = h.iter().filter(|&n| *n == ALIVE).count();
    return alive_n >= (write_concern - 1);
}



async fn send_message(msg_req: SecondaryMessageRequest, url: String, sec_id: usize, data_health_status: Data<HealthStatusList>) {
    let mut health_status: HealthStatus;
    let client = reqwest::Client::new();
    let mut retry_interval = time::interval(Duration::from_secs(5));
    loop {
        {
            health_status = data_health_status.read().unwrap()[sec_id];
        }
        if health_status == DEAD {
            retry_interval.tick().await;
            continue
        }
        let response = client.post(url.clone()).timeout(Duration::from_secs(15)).json(&msg_req).send().await;
        match response {
            // futures return w/o error
            Ok(response) => { if response.status() == StatusCode::OK {
                break;
            } },
            // futures returned with error
            _ => continue,
        }
    }
}

#[get("/public/health/")]
async fn get_sec_health(data_health_status: Data<HealthStatusList>) -> Result<HttpResponse, Box<dyn Error>> {
    let h  = data_health_status.read().unwrap().to_vec();
    let response_json = serde_json::to_string(&h)?;

    let response = HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_json);
    Ok(response)
}

async fn update_health_status(data: Data<HealthStatusList>) {
    let client = reqwest::Client::new();
    let health_checks = SECONDARY_URLS.map(|address| {
            let url = format!("http://{}/{}/", address, "private/health");
            client.get(url).timeout(Duration::from_millis(100)).send()
        });

    let responses = join_all(health_checks).await;

    // Why here? Because the code between await must implement Send trait https://stackoverflow.com/questions/66061722/why-does-holding-a-non-send-type-across-an-await-point-result-in-a-non-send-futu
    let mut v = data.write().unwrap();
    for (idx, response) in responses.iter().enumerate(){
        let mut status = ALIVE;
        if let Ok(response) = response {
            if response.status() != StatusCode::OK {
                status = DEAD;
            }
        }
        else {
            status = DEAD;
        }
        v[idx] = status;
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let messages_log = Data::new(RwLock::new(BTreeMap::<MessageID, Arc<Message>>::new()));
    let secondaries_health = Data::new(RwLock::new([DEAD; SECONDARY_URLS.len()]));
    let clone_sec_health = secondaries_health.clone();

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            update_health_status(clone_sec_health.clone()).await;
            interval.tick().await;
        }
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(post_message)
            .service(get_messages)
            .service(get_sec_health)
            .app_data(Data::clone(&messages_log))
            .app_data(Data::clone(&secondaries_health))
    })
    .bind(("0.0.0.0", 8080))?
    .run().await
}