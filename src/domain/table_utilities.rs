use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, Display};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TableName {
    UserTable,
    ConcertTable,
    ClaimedPasses,
    UnclaimedPasses,
    VenueTable
}


#[derive(Debug, Error)]
pub enum TableError {
    #[error("The table '{0}' os not supported")]
    TableNotSupported(String),
    #[error("Table '{0}' Could not be downcast")]
    TableDowncastError(String),
    #[error("Query '{0}' for table '{1}' could not be made")]
    QueryPrepError(String, String),
    #[error("Query could not be executed because of {0}")]
    QueryExecutionError(String),
    #[error("Row values missing in insert statement")]
    MissingRowValues,
    #[error("Table '{0}' not found in query map")]
    TableNameNotFound(String),
    #[error("table '{0}' not compatible with {1}")]
    RowValueMismatch(String, String),
}

