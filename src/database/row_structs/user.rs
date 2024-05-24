use std::fmt::{Debug, Formatter};
use uuid::Uuid;
use scylla::{FromRow, SerializeRow};
use crate::database::row_structs::Nameable;
use chrono::{Utc, DateTime};
use scylla::frame::value::CqlTimestamp;
use serde::Serialize;
use struct_iterable::Iterable;
use super::timestamp_serde;
use uuid::serde::simple;
use crate::database::{SelectQueries, SelectQueryChange};

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

#[derive(SerializeRow, Debug)]
pub struct UserConditions {
    pub user_uuid: Option<Uuid>
}

impl UserConditions {
    pub fn new(user_uuid: Uuid) -> Self {
        UserConditions { user_uuid: Some(user_uuid) }
    }
    pub fn conditional(&self) -> String {
        if self.user_uuid.is_some() {
            format!(" = {}", self.user_uuid.unwrap())
        } else {
            "".to_string()
        }

    }
}

#[derive(PartialEq, Eq, Hash, Iterable, Debug, Clone, Copy)]
pub struct UserSelectQueries {
    pub user_uuid: bool
}

impl From<&UserConditions> for UserSelectQueries {
    fn from(value: &UserConditions) -> Self {
        Self { user_uuid: value.user_uuid.is_some() }
    }
}

impl SelectQueryChange for UserConditions {
    fn get_enum(&self) -> SelectQueries {
        let inner: UserSelectQueries = self.into();
        SelectQueries::User(inner)
    }
}
