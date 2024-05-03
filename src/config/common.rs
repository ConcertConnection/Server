use serde::Deserialize;


use crate::server::ServerConfig;
use crate::database::DatabaseConfig

#[derive(Deserialize, Clone)]
pub struct CommonConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig
}