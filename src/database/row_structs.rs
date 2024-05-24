extern crate proc_macro;
mod user;
mod claimed_pass;
mod unclaimed_pass;
mod concert;
mod venue;

use std::fmt::Debug;
use std::hash::Hash;
use scylla::_macro_internal::SerializeRow;
use scylla::frame::value::CqlTimestamp;
use struct_iterable::Iterable;
pub use user::*;
pub use claimed_pass::*;
pub use concert::*;
pub trait Nameable {
    fn get_name(&self) -> String;
}


mod timestamp_serde {
    use super::*;
    use serde::{Serialize, Deserialize};

    pub fn serialize<S>(timestamp: &CqlTimestamp, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        let serialized_timestamp = timestamp.0;
        serialized_timestamp.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<CqlTimestamp, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        let serialized_timestamp = i64::deserialize(deserializer)?;
        Ok(CqlTimestamp { 0:serialized_timestamp })
    }
}


#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum SelectQueries {
    User(UserSelectQueries),
    Concert(ConcertSelectQueries),
    ClaimedPass(ClaimedPassSelectQueies)
}

pub trait SelectQueryChange: SerializeRow + Debug {
    fn get_enum(&self) -> SelectQueries;
}