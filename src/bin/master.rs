use std::collections::BTreeMap;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use actix_web::http::StatusCode;
use actix_web::{
    middleware::Logger,
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use env_logger;
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest;
use tokio;

use rlog::common::{append_message, get_messages, MessagesAppData};
use rlog::errors::WriteConcernNotSatisfiedError;
use rlog::structs::{MasterMessageRequest, Message, MessageID, SecondaryMessageRequest};

mod rlog;

const SECONDARY_URLS: [&'static str; 2] = ["localhost:8081", "localhost:8082"];
// const SECONDARY_URLS: [&str; 0] = [];

const SECONDARY_PATH: &'static str = "private/message";
static GLOBAL_MESSAGES_COUNT: AtomicUsize = AtomicUsize::new(0);

#[post("/public/message/")]
async fn post_message(
    data: MessagesAppData,
    request: web::Json<MasterMessageRequest>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let request = request.into_inner();
    let write_concern = request.wc.0;
    let message_id = MessageID(GLOBAL_MESSAGES_COUNT.fetch_add(1, Ordering::AcqRel));
    let message_ptr = Arc::clone(&request.msg_ptr);

    // append message on master
    append_message(data, request.msg_ptr, message_id.clone());

    // send asynchronous requests to the secondaries
    let mut req_futures = FuturesUnordered::new();
    SECONDARY_URLS.map(|address| {
        let msg_ptr = Arc::clone(&message_ptr);
        let id = message_id.clone();

        let future = tokio::spawn(async move {
            let url = format!("http://{}/{}/", address, SECONDARY_PATH);
            let client = reqwest::Client::new();
            let secondary_request = SecondaryMessageRequest { msg_ptr, id };
            client.post(url).json(&secondary_request).send().await
        });

        req_futures.push(future);
    });

    let mut nadded: usize = 1;
    loop {
        if nadded == write_concern {
            break;
        }
        match req_futures.next().await {
            Some(result) => {
                if let Ok(Ok(response)) = result{
                    nadded += (response.status() == StatusCode::OK) as usize;
                }
            }
            None => {
                break;
            }
        }
    }

    if nadded < write_concern {
        return Err(Box::new(WriteConcernNotSatisfiedError::new(
            write_concern,
            nadded,
        )));
    }

    Ok(HttpResponse::Ok().body("Success"))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app_data = Data::new(Mutex::new(BTreeMap::<MessageID, Arc<Message>>::new()));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(post_message)
            .service(get_messages)
            .app_data(Data::clone(&app_data))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
