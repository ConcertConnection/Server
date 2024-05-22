use serde::Deserialize;
use crate::config::email_client::EmailClientSettings;


use super::server::ServerConfig;
use super::database::DatabaseConfig;

#[derive(Deserialize, Clone, Debug)]
pub struct CommonConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub email_client: EmailClientSettings,
}