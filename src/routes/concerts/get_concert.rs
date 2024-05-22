use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use image::{DynamicImage, ImageFormat};
use image::io::Reader as ImageReader;
use std::io::Cursor;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc, serde::ts_seconds};



#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Concert {
    pub concert_uuid: Option<Uuid>,
    pub venue_uuid: Uuid,
    #[serde(with = "ts_seconds")]
    pub concert_date: DateTime<Utc>,
    pub artist_name: String,
    pub artist_image: Option<String>,
    pub artist_video: Option<String>,
    pub standby_passes: u16,
    pub secured_passes: u16
}


pub fn image_to_base64(img: &DynamicImage) -> String {
    let mut image_data: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut image_data), ImageFormat::Png)
        .unwrap();
    let res_base64 = general_purpose::STANDARD.encode(image_data);
    format!("data:image/png;base64,{}", res_base64)
}


pub async fn get_concert(concert_id: String) -> HttpResponse {
    let img = match ImageReader::open("Static/0922_Noise_Khruangbin_Courtesy_Jackie-Lee-Young.jpg") {
        Ok(img) => match img.decode() {
            Ok(img) => img,
            Err(_) => return HttpResponse::InternalServerError().finish()
        },
        Err(_) => return HttpResponse::InternalServerError().finish()
    };
    let fake_concert = Concert {
        concert_uuid:Uuid::new_v4(),
        venue_uuid: Uuid::new_v4(),
        concert_date: Utc::now(),
        artist_name: "khruangbin".to_string(),
        artist_image: Some(image_to_base64(&img)),
        artist_video: None,
        standby_passes: 22,
        secured_passes: 106
    };

    HttpResponse::Ok().json(fake_concert)

}