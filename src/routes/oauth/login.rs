use tracing::Instrument;
use actix_web::{HttpResponse, web};

pub async fn login(login: String) -> HttpResponse {
    HttpResponse::Ok().body("Some-user-id")
}