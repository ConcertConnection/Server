use serde::Deserialize;


use crate::server::ServerConfig;

#[derive(Clone)]
pub struct CommonConfig {
    pub server: ServerConfig
}