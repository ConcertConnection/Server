use tracing::Instrument;
use actix_web::{HttpResponse, web};
use crate::database::DatabaseConnection;

pub async fn release_pass(user_id: String, concert_id: String) -> HttpResponse {
    HttpResponse::Ok().finish()
}