use actix_web::{Responder, web};
use uuid::Uuid;
use crate::database::DatabaseConnection;
use crate::routes::Concert;

pub async fn new_concert(concert: web::Json<Concert>, db_conncection: web::Data<DatabaseConnection>) -> impl Responder {
    if concert.concert_uuid.is_none() {
        concert.concert_uuid = Some(Uuid::new_v4());
    }

    db_conncection.execute("concert_table".into(), "insert", )

}