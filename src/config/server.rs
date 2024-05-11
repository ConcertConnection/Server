use serde::Deserialize;
use serde_this_or_that::as_i64;

#[derive(Deserialize, Clone, Debug)]
pub struct  ServerConfig {
    host: String,
    #[serde(deserialize_with = "as_i64")]
    port: i64
}