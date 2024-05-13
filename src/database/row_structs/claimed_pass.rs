use uuid::Uuid;
use scylla::{FromRow, SerializeRow};
use super::Nameable;
use chrono::{Utc, DateTime};
use scylla::frame::value::CqlTimestamp;

#[derive(FromRow, SerializeRow, Eq, PartialEq, Debug)]
pub struct ClaimedPass {
    ticket_uuid: Uuid,
    concert_uuid: Uuid,
    user_uuid: Uuid,
    venue_uuid: Uuid,
    concert_date: CqlTimestamp,
    venue_name: String,
    artist_name: String,
    standby: bool
}


impl Nameable for ClaimedPass {
    fn get_name(&self) -> String {
        String::from("claimed_pass")
    }
}


impl ClaimedPass {
    pub fn new(
        ticket_uuid: Uuid,
        concert_uuid: Uuid,
        user_uuid: Uuid,
        venue_uuid: Uuid,
        concert_date: DateTime<Utc>,
        venue_name: String,
        artist_name: String,
        standby: bool
    ) -> Self {
        let concert_date = CqlTimestamp(concert_date.timestamp_millis());

        Self {
            ticket_uuid,
            concert_uuid,
            user_uuid,
            venue_uuid,
            concert_date,
            venue_name,
            artist_name,
            standby
        }
    }
}