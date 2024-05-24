use claims::{assert_err, assert_ok, assert_some};
use crate::helpers::spawn_app;
use concert_connect_server::routes::{Concert, NewConcertConfirmation};
use concert_connect_server::routes::parse_image_string;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use fake::Fake;
use uuid::Uuid;


#[tokio::test]
async fn new_concert_inserts_a_retrievable_concert_with_no_image() {
    let app = spawn_app().await;

    let mut concert = Concert {
        concert_uuid: None,
        venue_uuid: Uuid::new_v4(),
        concert_date: Utc::now(),
        secured_passes: 22,
        artist_name: "huge guys".to_string(),
        artist_image: None,
        artist_video: None,
        standby_passes: 12,
        artist_bio: Some("I really like this bio".to_string())
    };

    let uuid_response = app.new_concert(&concert).await;
    println!("{:#?}", uuid_response);
    let body: NewConcertConfirmation = uuid_response.json()
        .await
        .unwrap();
    println!("BODY: {:#?}", &body);
    let uuid = &body.concert_uuid;

    concert.concert_uuid = Some(uuid.clone());

    let concert_response = app.get_concert(uuid.to_string())
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(concert, concert_response)



}