pub use common::*;
use environment::*;
use config::{
    Config,
    File,
    ConfigError
};
pub mod authenticator;
pub mod common;
mod server;
pub mod database;
mod environment;
mod email_client;
pub fn load_configuration() -> Result<CommonConfig, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to retrieve current directory");
    let config_dir = base_path.join("config");

    let env_config: Environment = std::env::var("CC_ENV")
        .unwrap_or_else(|_| "Dev".into())
        .try_into()
        .expect("Could not find the environment variable");

    let env_filename = format!("{}.yaml", env_config.as_str());

    let config = Config::builder()
        .add_source(File::from(config_dir.join("general.yaml")))
        .add_source(File::from(config_dir.join(env_filename)))
        .build()?;

    config.try_deserialize::<CommonConfig>()
}


#[cfg(test)]
mod test_config {
    use crate::config::load_configuration;

    #[test]
    fn test_local_configuration() {
        std::env::set_var("CC_ENV", "Dev");
        let config = load_configuration();
        println!("{:?}", config);
        assert!(config.is_ok(), "{}", config.unwrap_err());
        let config = config.unwrap();
        assert_eq!(config.database.database_addr(), "127.0.0.1:9042".to_string());
        assert_eq!(config.database.keyspace(), "concert_connect".to_string());
        assert_eq!(config.database.tables.user_table.name, "user_table");
        // assert_eq!(
        //     config.database.tables.user_table.columns,
        //     vec![
        //         ("user_uuid","uuid"),
        //         ("first_name","text"),
        //         ("last_name","text"),
        //         ("email","boolean"),
        //         ("active","boolean"),
        //         ("paused","boolean"),
        //         ("sign_up_date","timestamp")
        //     ],
        //     "{}",
        //     config.database.tables.user_table.columns
        //         .iter()
        //         .map(|entry| (&entry.0[..]).to_owned() + &entry.1[..])
        //         .collect()
        //         .join(", ")
        // )

    }

    #[test]
    fn test_production_configuration() {
        std::env::set_var("CC_ENV", "prod");
        let config = load_configuration();
        assert!(config.is_ok())
    }
}
