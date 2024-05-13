use uuid::Uuid;
use scylla::{FromRow, SerializeRow};
use crate::database::row_structs::Nameable;
use chrono::{Utc, DateTime};
use scylla::frame::value::CqlTimestamp;

#[derive(FromRow, SerializeRow, Eq, PartialEq, Debug)]
pub struct User {
    user_uuid: Uuid,
    first_name: String,
    last_name: String,
    email: String,
    active: bool,
    paused: bool,
    sign_up_date: CqlTimestamp
}

impl Nameable for User {
    fn get_name(&self) -> String {
        String::from("user")
    }
}

impl User {
    pub fn new(
        user_uuid: Uuid,
        first_name: String,
        last_name: String,
        email: String,
        active: bool,
        paused: bool,
        sign_up_date: Option<DateTime<Utc>>
    ) -> Self {

        let sign_up_date = sign_up_date.unwrap_or_else(|| Utc::now()).timestamp_millis();
        let sign_up_date = CqlTimestamp(sign_up_date);
        User {
            user_uuid,
            first_name,
            last_name,
            email,
            active,
            paused,
            sign_up_date
        }
    }
}