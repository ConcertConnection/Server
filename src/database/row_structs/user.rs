use uuid::Uuid;
use scylla::{FromRow, SerializeRow};
use crate::database::row_structs::Nameable;
use chrono::{Utc, DateTime};
use scylla::frame::value::CqlTimestamp;
use serde::Serialize;
use super::timestamp_serde;
use uuid::serde::simple;
#[derive(FromRow, SerializeRow, Eq, PartialEq, Debug, Serialize)]
pub struct User {
    #[serde(with="simple")]
    user_uuid: Uuid,
    first_name: String,
    last_name: String,
    email: String,
    active: bool,
    paused: bool,
    #[serde(with= "timestamp_serde")]
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

#[derive(SerializeRow)]
pub struct UserConditions {
    pub(crate) user_uuid: Uuid
}

impl UserConditions {
    pub fn new(user_uuid: Uuid) -> Self {
        UserConditions { user_uuid }
    }
    pub fn conditional(&self) -> String {
        format!(" = {}", self.user_uuid)
    }
}