use tracing::Instrument;
use actix_web::{HttpResponse, web};

pub async fn send_sign_up_email(user_id: String) -> HttpResponse {
    HttpResponse::Ok().finish()
}