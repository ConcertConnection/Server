use std::fmt::format;
pub use common::*;
use config::{
    Config,
    File,
    Environment,
    ConfigError
};
mod authenticator;
mod common;
mod server;
mod database;
mod environment;
pub fn load_configuration() -> Result<CommonConfig, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to retrieve current directory");
    let config_dir = base_path.join("config");

    let env_config: Environment = std::env::var("CC_ENV")
        .unwrap_or_else(|_| "local".into())
        .expect("Could not find the environment variable");

    let env_filename = format!("{}.yaml", env_config.as_string());

    let config = Config::builder()
        .add_source(File::from(config_dir.join("general.yaml")))
        .add_source(File::from(config_dir.join(env_filename)))
        .build()?;

    config.try_deserialize::<CommonConfig>()
}