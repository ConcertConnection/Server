use claims::{assert_err, assert_ok, assert_some};
use tracing_subscriber::filter::FilterExt;
use crate::helpers::spawn_app;
use concert_connect_server::routes::Concert;
use concert_connect_server::routes::parse_image_string;
use base64::{engine::general_purpose, Engine as _};


#[tokio::test]
async fn get_concerts_returns_a_200_for_valid_concert_id() {
    let app = spawn_app().await;
    let concert_id = "concert_id=Some_Concert_id";

    let response = app.get_concert(concert_id.to_string())
        .await;
    assert_eq!(200, response.status().as_u16())
}

#[tokio::test]
async fn get_concerts_returns_an_image() {
    let app = spawn_app().await;
    let concert_id = "Some_Concert_id";
    let response: Concert = app.get_concert(concert_id.to_string())
        .await
        .json()
        .await
        .unwrap();
    let img = &response.artist_image;
    assert_some!(&img);
    let img = parse_image_string(&img.as_ref().unwrap());
    let img = general_purpose::STANDARD.decode(img);
    assert_ok!(img);
}
