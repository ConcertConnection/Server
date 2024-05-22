use tracing::Instrument;
use actix_web::{HttpResponse, web};
use crate::database::DatabaseConnection;

struct PassInfoForm {
    user_id: String,
    concert_id: String
}


#[tracing::instrument(
    name = "Claiming pass for subscriber",
    skip(pass_info_form, db_connection),
    fields(
        member_id = %pass_info_form.user_id,
        concert_id = %pass_info_form.concert_id
    )
)]
pub async fn claim_pass(pass_info_form: web::Form<PassInfoForm>, db_connection: web::Data<DatabaseConnection>) -> HttpResponse {
    HttpResponse::Ok().body("Secured")
}


