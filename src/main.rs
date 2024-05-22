use concert_connect_server::config::load_configuration;
use concert_connect_server::telemetry::{get_subscriber, init_subscriber};
use concert_connect_server::startup::Application;
#[tokio::main]
async fn main() -> Result<(), std::io::Error>{
    let subscriber = get_subscriber("Server".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = load_configuration().expect("Could not load configuration");
    let application = Application::build(configuration).await?;

    application.run_until_stopped().await?;
    Ok(())
}
