use uuid::Uuid;
use scylla::{FromRow, SerializeRow};
use super::Nameable;
use chrono::{Utc, DateTime};
use scylla::frame::value::CqlTimestamp;
use serde::{Deserialize, Serialize};
use struct_iterable::Iterable;
use crate::database::row_structs::timestamp_serde;
use uuid::serde::simple;
use crate::database::{SelectQueries, SelectQueryChange};

#[derive(FromRow, SerializeRow, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct ClaimedPass {
    #[serde(with="simple")]
    ticket_uuid: Uuid,
    #[serde(with="simple")]
    concert_uuid: Uuid,
    #[serde(with="simple")]
    user_uuid: Uuid,
    #[serde(with="simple")]
    venue_uuid: Uuid,
    #[serde(with= "timestamp_serde")]
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

#[derive(SerializeRow, Debug)]
pub struct ClaimedPassConditions {
    ticket_uuid: Option<Uuid>,
    concert_uuid: Option<Uuid>,
    user_uuid: Option<Uuid>
}

impl ClaimedPassConditions {
    pub fn new(ticket_uuid: Option<Uuid>, concert_uuid: Option<Uuid>, user_uuid: Option<Uuid>) -> Self {
        Self { ticket_uuid, concert_uuid, user_uuid }
    }
}

#[derive(PartialEq, Eq, Hash, Iterable, Debug, Clone, Copy)]
pub struct ClaimedPassSelectQueies {
    ticket_uuid: bool,
    concert_uuid: bool,
    user_uuid: bool
}

impl From<&ClaimedPassConditions> for ClaimedPassSelectQueies {
    fn from(value: &ClaimedPassConditions) -> Self {
        Self {
            ticket_uuid: value.ticket_uuid.is_some(),
            concert_uuid: value.concert_uuid.is_some(),
            user_uuid: value.user_uuid.is_some()
        }
    }
}

impl SelectQueryChange for ClaimedPassConditions {
    fn get_enum(&self) -> SelectQueries {
        let inner: ClaimedPassSelectQueies = self.into();
        SelectQueries::ClaimedPass(inner)
    }
}