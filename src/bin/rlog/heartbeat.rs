use std::error::Error;
use std::sync::RwLock;
use std::time::Duration;

use actix_web::{get, http::header::ContentType, http::StatusCode, HttpResponse, web::Data};
use futures::future::join_all;
use log::info;
use serde::{Deserialize, Serialize};

use super::config::SECONDARY_URLS;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum HealthStatus {
    ALIVE,
    DEAD,
}

type HealthStatusList = [HealthStatus; SECONDARY_URLS.len()];
pub type HealthStatusListAppData = Data<RwLock<HealthStatusList>>;


pub async fn update_health_status(data: HealthStatusListAppData) {
    info!("Checking health!");
    let client = reqwest::Client::new();
    let health_checks = SECONDARY_URLS.map(|address| {
        let url = format!("http://{}/{}/", address, "private/health");
        client.get(url).timeout(Duration::from_millis(100)).send()
    });
    let responses = join_all(health_checks).await;

    // Why here? Because the code between await must implement Send trait
    // https://stackoverflow.com/questions/66061722/why-does-holding-a-non-send-type-across-an-await-point-result-in-a-non-send-futu
    let mut v = data.write().unwrap();
    for (idx, response) in responses.iter().enumerate() {
        v[idx] = match response {
            Ok(response) if response.status() == StatusCode::OK => HealthStatus::ALIVE,
            _ => HealthStatus::DEAD
        };
    }
}

#[get("/public/health/")]
pub async fn get_secondaries_health(
    data_health_status: HealthStatusListAppData) -> Result<HttpResponse, Box<dyn Error>> {
    let health: HealthStatusList;
    {
        // make copy here since array of enum has Copy trait implemented
        health = *data_health_status.read().unwrap();
    }
    let response_json = serde_json::to_string(&health)?;

    let response = HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_json);
    Ok(response)
}