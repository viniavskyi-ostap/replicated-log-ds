// use std::{ops::Deref, path};
use std::{sync::Mutex, process::Output, string, collections::HashMap, hash::Hash};

use actix::fut::future;
use actix_web::{App, HttpServer, middleware::Logger, web::{self, Data}, HttpResponse, post };
use actix_web::http::{StatusCode};
use futures::{future::join_all, Future, stream::FuturesUnordered, StreamExt};
use reqwest::{self, Response};
// use actix-rt;
// use actix_web::http::{header::ContentType, StatusCode};
use env_logger;
// use tokio::task{spawn}
use tokio::spawn;
// #[path = "../messages.rs"]
mod rlog;
use rlog::messages::{Message, MessageID, get_messages, save_message};
use serde::de::value::Error;
use::std::sync::atomic::{AtomicUsize, Ordering};


const SECONDARY_URLS: [&str; 2] =["127.0.0.1:8081", "127.0.0.1:8082"];
const NUM_SECONDARIES: usize = SECONDARY_URLS.len();
const SECONDARY_PATH: &str = "private/message";
static GLOBAL_MESSAGES_COUNT: AtomicUsize = AtomicUsize::new(0);


#[post("/public/message/")]
async fn post_message(data: Data<Mutex<HashMap<MessageID, Message>>>, req: web::Json<Message>) -> HttpResponse {
    let msg = req.into_inner();
    let write_concern = msg.concern();
    let mut msg_to_send = msg.clone();
    msg_to_send.set_id(GLOBAL_MESSAGES_COUNT.fetch_add(1, Ordering::AcqRel));
    let save_reponse = save_message(data, msg);

    let client = reqwest::Client::new();
    let mut responces = FuturesUnordered::new();
    
    let closure = |address: String| {
        
        let msg = msg_to_send.clone();
        let client = client.clone();
        let addr = address.clone();
        
        responces.push(
            spawn(async move {
            let url = format!("http://{}/{}/", addr, SECONDARY_PATH);
             client.post(url).json(&msg).send().await 
        }))
    };

    SECONDARY_URLS.map(|address| closure(address.to_string()));

    if let Err(_) = save_reponse {return HttpResponse::InternalServerError().body("body"); }

    let mut nadded=1;
    loop{
        if nadded == write_concern {
            
            break;
        }
        match responces.next().await {
            Some(result) => {
                if let Ok(response) = result {
                    if response.is_ok() {
                        nadded += 1;    
                    }
                } 
            }
            None => {break;}
        }
    }

    if nadded < write_concern{
        HttpResponse::InternalServerError().body("body")
    }else {
        HttpResponse::Ok().body("body")
    }

}

// async fn handle_responces<T>(resps:T, wc:i32) -> HttpResponse
// {
//     let mut nadded = 1;
//     loop {
//         match resps.next().await {
//             Some(result) => 
//             {

//             }
//             None => {break;}
//         }
//     }
// }

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let msg_vec: Data<Mutex<HashMap<MessageID, Message>>> = Data::new(Mutex::new(HashMap::new()));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(post_message)
            .service(get_messages)
            .app_data(Data::clone(&msg_vec))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}