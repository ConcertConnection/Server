use tracing::Instrument;
use actix_web::{HttpResponse, web};
use crate::database::DatabaseConnection;
use uuid::Uuid;

pub async fn get_user_passes(user_id: String) -> HttpResponse {
    let user_id = Uuid::parse_str(&user_id);
    HttpResponse::Ok().finish()
}