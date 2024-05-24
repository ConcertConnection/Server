use std::net::TcpListener;
use std::sync::Mutex;
use actix_web::dev::Server;
use tracing_actix_web::TracingLogger;
use actix_web::{App, HttpServer, web};
use crate::email_client::EmailClient;
use crate::routes;
use crate::config::database::DatabaseConfig;
use crate::config::CommonConfig;
use crate::database::DatabaseConnection;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: CommonConfig) -> Result<Self, std::io::Error> {
        let connection_pool = DatabaseConnection::build(&configuration)
            .await.expect("Could not connect to the database");
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Found invalid sender email.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout
        );
        let address = format!(
            "{}:{}",
            configuration.server.host, configuration.server.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let app_state = AppState {db_pool: Mutex::new(connection_pool), email_client};
        let server = run(listener, app_state)?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct AppState {
    pub db_pool: Mutex<DatabaseConnection>,
    pub email_client: EmailClient
}



pub fn run(
    listener: TcpListener,
    app_state: AppState
) -> Result<Server, std::io::Error> {
    let app_state = web::Data::new(app_state);
    let server = HttpServer::new(
        move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/health_check", web::get().to(routes::health_check))
                .route("/get_concert", web::get().to(routes::get_concert))
                .route("/new_concert", web::post().to(routes::new_concert))
                .app_data(app_state.clone())
        }
    )
        .listen(listener)?
        .run();
    Ok(server)
}
