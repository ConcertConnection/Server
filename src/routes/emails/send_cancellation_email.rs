
use tracing::Instrument;
use actix_web::{HttpResponse, web};

pub async fn send_cancellation_email(user_id: String) -> HttpResponse {
    HttpResponse::Ok().finish()
}