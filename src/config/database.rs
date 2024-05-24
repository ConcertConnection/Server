use serde::Deserialize;
use struct_iterable::Iterable;
use crate::domain::TableName;

#[derive(Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    database_addr: String,
    pub(crate) keyspace: String,
    pub tables: Tables
}

impl DatabaseConfig {
    pub fn database_addr(&self) -> &str {
        &self.database_addr
    }
    pub fn keyspace(&self) -> &str {
        &self.keyspace
    }
    pub fn set_keyspace(&mut self, new_keyspace: String) {
        self.keyspace = new_keyspace
    }
}


#[derive(Deserialize, Clone, Debug, Iterable)]
pub struct Tables {
    pub user_table: TableConfig,
    pub claimed_passes: TableConfig,
    pub unclaimed_passes: TableConfig,
    pub concert_table: TableConfig,
    pub venue_table: TableConfig,
}


#[derive(Deserialize, Clone, Debug)]
pub struct TableConfig {
    pub name: TableName,
    pub columns: Vec<Column>,
    pub primary_key: String
}


#[derive(Deserialize, Clone, Debug)]
pub struct Column {
    pub name: String,
    pub dtype: String
}