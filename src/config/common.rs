use serde::Deserialize;


use super::server::ServerConfig;
use super::database::DatabaseConfig;

#[derive(Deserialize, Clone, Debug)]
pub struct CommonConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig
}