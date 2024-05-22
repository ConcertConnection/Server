use fake::Fake;
use fake::faker::company::en::CompanyName;
use fake::faker::internet::en::DomainSuffix;
use fake::locales::EN;
use once_cell::sync::Lazy;
use tracing_subscriber::fmt::format;
use uuid::Uuid;
use concert_connect_server::config::{load_configuration};
use concert_connect_server::telemetry::{get_subscriber, init_subscriber};
use concert_connect_server::email_client::EmailClient;
use concert_connect_server::startup::Application;
use concert_connect_server::database::DatabaseConnection;


static TRACING: Lazy<()> = Lazy::new( || {
    let default_filter_level = "info".to_string();
    let subscriber_name = "tests".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("tests".into(), "debug".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("tests".into(), "debug".into(), std::io::sink);
        init_subscriber(subscriber);
    }

});



pub struct TestApp {
    pub address: String,
    pub db_pool: DatabaseConnection
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn get_concert(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!("{}/get_concert", &self.address))
            .header("Content-Type", "application/x-ww-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

fn fake_keyspace() -> String {
    let keyspace_name: String = format!(
        "{}{}{}",
        CompanyName().fake::<String>(),
        DomainSuffix().fake::<String>(),
        rand::random::<u16>()
    ).to_lowercase().replace(" ", "");
    keyspace_name
}

pub(crate) async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = load_configuration().expect("Failed to read config.");
        c.database.set_keyspace(fake_keyspace());
        c.server.port = 0;
        c
    };

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");

    let address = format!("http://0.0.0.0:{}", application.port());

    let _ = tokio::spawn(application.run_until_stopped());
    let database_connect = DatabaseConnection::build(&configuration)
        .await.expect("Cold not connect to database");

    TestApp{
        address,
        db_pool: database_connect
    }
}


