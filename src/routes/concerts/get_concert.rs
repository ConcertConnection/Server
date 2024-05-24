use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use image::{DynamicImage, guess_format, ImageFormat};
use image::io::Reader as ImageReader;
use std::io::Cursor;
use std::path::Path;
use std::sync::Mutex;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc, serde::ts_seconds};
use tracing::instrument;
use crate::database::{Concert as ConcertDB, ConcertConditions, DatabaseConnection};
use crate::startup::AppState;


#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all="camelCase")]
pub struct Concert {
    pub concert_uuid: Option<Uuid>,
    pub venue_uuid: Uuid,
    #[serde(with = "ts_seconds")]
    pub concert_date: DateTime<Utc>,
    pub artist_name: String,
    pub artist_image: Option<String>,
    pub artist_video: Option<String>,
    pub artist_bio: Option<String>,
    pub standby_passes: i32,
    pub secured_passes: i32
}

#[instrument(
    name="Converting-Image-encodings"
)]
pub fn image_to_base64(img: &DynamicImage, image_format: String) -> String {
    let mut image_data: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut image_data), ImageFormat::from_extension(image_format).unwrap())
        .unwrap();
    let res_base64 = general_purpose::STANDARD.encode(image_data);
    format!("data:image/png;base64,{}", res_base64)
}


#[instrument(
    name="Get-Concert",
    skip(app_state)
)]
pub async fn get_concert(concert_id: String, app_state: web::Data<AppState>) -> HttpResponse {

    let mut query_results = app_state.db_pool.lock().unwrap()
        .execute::<ConcertDB, ConcertConditions>(
            &"concert_table".to_string(),
            &mut "select".to_string(),
            None,
            Some(
                &ConcertConditions::new(
                    Some(Uuid::parse_str(&concert_id).unwrap()),
                    None
                )
            )
        )
        .await;

    let query_results = match query_results {
        Ok(query_result) => query_result.rows_typed::<ConcertDB>().unwrap(),
        Err(e) => return HttpResponse::BadRequest().body(e.to_string())
    };

    let path = "Static/0922_Noise_Khruangbin_Courtesy_Jackie-Lee-Young.jpg";
    let extention = if let Some(extension) = Path::new(path).extension().and_then(|ext| ext.to_str()) {
        extension.to_string()
    } else {
        return HttpResponse::InternalServerError().finish();
    };
    let img = match ImageReader::open(path) {
        Ok(img) => match img.decode() {
            Ok(img) => img,
            Err(_) => return HttpResponse::InternalServerError().finish()
        },
        Err(_) => return HttpResponse::InternalServerError().finish()
    };
    let fake_concert = Concert {
        concert_uuid:Some(Uuid::new_v4()),
        venue_uuid: Uuid::new_v4(),
        concert_date: Utc::now(),
        artist_name: "khruangbin".to_string(),
        artist_image: Some(image_to_base64(&img, extention)),
        artist_video: None,
        standby_passes: 22,
        secured_passes: 106,
        artist_bio: Some("This a really great bio!".to_string())
    };

    HttpResponse::Ok().json(fake_concert)

}