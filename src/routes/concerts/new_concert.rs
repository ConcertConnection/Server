use std::io::Cursor;
use std::path::Path;
use actix_web::{HttpResponse, Responder, web};
use base64::{engine::general_purpose, Engine as _};
use image::{ColorType, GenericImageView, ImageError, ImageFormat};
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;
use crate::database::{ConcertConditions, DatabaseConnection};
use crate::routes::Concert as ConcertWeb;
use crate::database::Concert;
use image::io::Reader as ImageReader;
use serde::{Deserialize, Serialize};
use crate::startup::AppState;


#[derive(Serialize, Deserialize, Debug)]
pub struct NewConcertConfirmation {
    pub concert_uuid: Uuid,
    artist_image: String,
    artist_video: String,
    artist_bio: String
}

impl From<Concert> for NewConcertConfirmation {
    fn from(value: Concert) -> Self {
        NewConcertConfirmation{
            concert_uuid: value.concert_uuid,
            artist_bio: value.artist_bio,
            artist_video: value.artist_video,
            artist_image: value.artist_image
        }
    }
}

#[instrument(
    name="Inserting New Concert",
    skip(app_state)
)]
pub async fn new_concert(mut concert: web::Json<ConcertWeb>, app_state: web::Data<AppState>) -> impl Responder {
    if concert.concert_uuid.is_none() {
        concert.concert_uuid = Some(Uuid::new_v4());
    }

    if concert.artist_image.is_some() {
        let artist_image = concert.artist_image.as_ref().unwrap();
        if artist_image.starts_with("data:image/png;base64") {
            process_image(parse_image_string(artist_image), &concert.artist_name)
        } else {
            process_image(artist_image.as_str(), &concert.artist_name)
        };
    }

    let concert: Concert = concert.0.into();

    match app_state.db_pool.lock().unwrap().execute::<Concert, ConcertConditions>(&"concert_table".to_string(), &mut "insert".to_string(), Some(&concert), None).await {
        Ok(_) => {
            let return_info: NewConcertConfirmation = concert.into();
            HttpResponse::Ok().json(return_info)
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}



#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Base64 decoding failed")]
    Base64DecodingFailed,
    #[error("Failed to guess image format")]
    ImageFormatGuessFailed,
    #[error("Parse image string failed")]
    ParseImageStringFailed,
}

#[instrument(
name="Getting image information",
skip(buffer)
)]
fn get_image_dimensions_and_color_format(buffer: &[u8]) -> Result<((u32, u32), ColorType), ImageError> {
    let cursor = Cursor::new(buffer);
    let img = ImageReader::new(cursor)
        .with_guessed_format()?
        .decode()?;
    let dimensions = img.dimensions();
    let color = img.color();
    Ok((dimensions, color))
}

#[instrument(
name="decoding image",
skip(image_data)
)]
fn decode_base64_image(image_data: &str) -> Result<Vec<u8>, ConversionError> {
    general_purpose::STANDARD
        .decode(image_data)
        .map_err(|_| ConversionError::Base64DecodingFailed)
}

#[instrument(
name="Guess image format"
skip(image_buff)
)]
fn guess_image_format(image_buff: &[u8]) -> Result<ImageFormat, ConversionError> {
    image::guess_format(image_buff)
        .map_err(|_| ConversionError::ImageFormatGuessFailed)
}

#[instrument(
name="Save Image"
skip(buffer, color_type)
)]
fn save_image(buffer: &[u8], path: &Path, width: u32, height: u32, color_type: ColorType) -> Result<(), ImageError> {
    image::save_buffer(path, buffer, width, height, color_type)
}

#[instrument(
    name="Processing Image Data",
    skip(image_data)
)]
fn process_image(image_data: &str, artist_name: &str) -> String {
    let Ok(decoded_image) = decode_base64_image(image_data) else { return "None".to_string() };
    let Ok(image_format) = guess_image_format(&decoded_image) else { return "None".to_string() };
    let image_extension = match image_format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpeg",
        _ => "unknown",
    };
    let image_path = format!("Static/{}.{}", artist_name, image_extension);
    let Ok(info) = get_image_dimensions_and_color_format(&decoded_image) else { return "None".to_string()};
    let ((width, height), color_type) = info;
    let Ok(_) = save_image(&decoded_image, Path::new(&image_path), width, height, color_type) else { return "None".to_string() };
    image_path
}
pub fn parse_image_string(img: &String) -> &str {
    let index = img.find(",").unwrap() + 1;
    let img = &img[index..];
    img
}