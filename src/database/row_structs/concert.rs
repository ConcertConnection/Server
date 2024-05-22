use std::io::Cursor;
use std::path::Path;
use base64::{engine::general_purpose, Engine as _};
use image::{ColorType, GenericImageView, ImageError, ImageFormat};
use scylla::frame::value::CqlTimestamp;
use uuid::Uuid;
use crate::routes::Concert as ConcertWeb;
use crate::telemetry::get_subscriber;
use image::io::Reader as ImageReader;
use thiserror::Error;


pub struct Concert {
    concert_uuid: Uuid,
    venue_uuid: Uuid,
    concert_date: CqlTimestamp,
    artist_name: String,
    artist_image: String,
    artist_video: String,
    standby_passes: u16,
    secured_passes: u16
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Base64 decoding failed")]
    Base64DecodingFailed,
    #[error("Failed to guess image format")]
    ImageFormatGuessFailed,
    #[error("Image dimensions or color format extraction failed: {0}")]
    ImageProcessingFailed(#[from] ImageError),
    #[error("Image saving failed: {0}")]
    ImageSavingFailed(#[from] ImageError),
    #[error("Parse image string failed")]
    ParseImageStringFailed,
}

fn get_image_dimensions_and_color_format(buffer: &[u8]) -> Result<((u32, u32), ColorType), ImageError> {
    let cursor = Cursor::new(buffer);
    let img = ImageReader::new(cursor)
        .with_guessed_format()?
        .decode()?;
    let dimensions = img.dimensions();
    let color = img.color();
    Ok((dimensions, color))
}

fn decode_base64_image(image_data: &str) -> Result<Vec<u8>, ConversionError> {
    general_purpose::STANDARD
        .decode(image_data)
        .map_err(|_| ConversionError::Base64DecodingFailed)
}

fn guess_image_format(image_buff: &[u8]) -> Result<ImageFormat, ConversionError> {
    image::guess_format(image_buff)
        .map_err(|_| ConversionError::ImageFormatGuessFailed)
}

fn save_image(buffer: &[u8], path: &Path, width: u32, height: u32, color_type: ColorType) -> Result<(), ImageError> {
    image::save_buffer(path, buffer, width, height, color_type)
}

fn process_image(image_data: &str, artist_name: &str) -> Result<String, ConversionError> {
    let decoded_image = decode_base64_image(image_data)?;
    let image_format = guess_image_format(&decoded_image)?;
    let image_extension = match image_format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpeg",
        _ => "unknown",
    };
    let image_path = format!("Static/{}.{}", artist_name, image_extension);
    let ((width, height), color_type) = get_image_dimensions_and_color_format(&decoded_image)?;
    save_image(&decoded_image, Path::new(&image_path), width, height, color_type)?;
    Ok(image_path)
}

impl TryFrom<ConcertWeb> for Concert {
    type Error = ConversionError;

    fn try_from(value: ConcertWeb) -> Result<Self, Self::Error> {
        let concert_uuid = value.concert_uuid.unwrap_or_else(Uuid::new_v4);

        let artist_image = match value.artist_image {
            Some(image_data) => {
                if image_data.starts_with("data:image/png;base64") {
                    process_image(parse_image_string(&Some(image_data))?, &value.artist_name)
                } else {
                    process_image(image_data.as_str(), &value.artist_name)
                }?
            },
            None => "None".to_string(),
        };

        let artist_video = "None".to_string();

        Ok(Concert {
            concert_uuid,
            venue_uuid: value.venue_uuid,
            concert_date: CqlTimestamp(value.concert_date.timestamp_millis()),
            artist_name: value.artist_name,
            artist_image,
            artist_video,
            secured_passes: value.secured_passes,
            standby_passes: value.standby_passes,
        })
    }
}
pub fn parse_image_string(img: &Option<String>) -> &'static str {
    let img = img.as_ref().unwrap();
    let index = img.find(",").unwrap() + 1;
    let img = &img[index..];
    img
}
