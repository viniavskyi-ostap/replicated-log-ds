use actix_web::{get, post, web, App, HttpServer, Responder};
use reqwest;
use futures::future::join_all;

const SECONDARY_URLS: [&str; 2] = ["127.0.0.1:8081", "127.0.0.1:8082"];
const SECONDARY_PATH: &str = "private/message";

#[get("/message")]
async fn get_messages() -> impl Responder {
    "hello"
}

#[post("/message/{msg}")]
async fn post_message(path: web::Path<String>) -> impl Responder {
    let msg = path.into_inner();

    let client = reqwest::Client::new();

    let responses = SECONDARY_URLS.map(|address| {
        let url = format!("http://{}/{}/{}",address, SECONDARY_PATH, msg);
        println!("{}", url);
        client.post(url).send()
    });
    let responses = join_all(responses).await;

    for response in responses {
        let text = match response {
            Ok(response) => {
                match response.text().await {
                    Ok(text) => text,
                    Err(e) => e.to_string()
                }
            }
            Err(e) => e.to_string()
        };
        println!("sent");
    }
    "hello"
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_messages)
            .service(post_message)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}