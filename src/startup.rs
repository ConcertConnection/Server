use std::net::TcpListener;
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
    server: Server
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
        let server = run(listener, connection_pool, email_client)?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}



pub fn run(
    listener: TcpListener,
    db_pool: DatabaseConnection,
    email_client: EmailClient
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(
        move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/health_check", web::get().to(routes::health_check))
                .route("/get_concert", web::get().to(routes::get_concert))
                .app_data(db_pool.clone())
                .app_data(email_client.clone())
        }
    )
        .listen(listener)?
        .run();
    Ok(server)
}
