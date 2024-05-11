use std::fmt::format;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    pub database_addr: String,
    pub keyspace: String,
    pub tables: Tables
}

#[derive(Deserialize, Clone, Debug)]
pub struct Tables {
    pub user_table: TableConfig,
    pub claimed_pass_table: TableConfig,
    pub unclaimed_pass_table: TableConfig,
    pub concert_table: TableConfig,
    pub venue_table: TableConfig,
}


#[derive(Deserialize, Clone, Debug)]
pub struct TableConfig {
    pub name: String,
    pub columns: Vec<Column>,
    pub primary_key: String
}


#[derive(Deserialize, Clone, Debug)]
pub struct Column {
    pub name: String,
    pub dtype: String
}