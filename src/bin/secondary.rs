use actix_web::{get, post, web::{self, Data}, App, HttpServer, Responder};
use std::sync::Mutex;


#[post("/private/message/{msg}")]
async fn get_message(data: Data<Mutex<Vec<String>>>, path: web::Path<String>) -> impl Responder {
    let msg = path.into_inner();
    data.lock().unwrap().push(msg);
    let v = data.lock().unwrap().clone();
    dbg!(v);
    "test"
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let msg_vec: Data<Mutex<Vec<String>>> = Data::new(Mutex::new(vec![]));

    HttpServer::new(move || {
        App::new()
            .service(get_message)
            .app_data(Data::clone(&msg_vec))
    })
        .bind(("127.0.0.1", 8081))?
        .run()
        .await
}