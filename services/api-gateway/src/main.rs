use api_gateway::{auth::jwt::JwtValidator, client::ServiceClient};
use api_gateway::config::get_config;
use telemetry::{get_subscriber, init_subscriber};
use std::net::TcpListener;

use api_gateway::startup::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("api-gateway".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_config().unwrap();

    let client = ServiceClient::new(config.services);

    let jwt_validator = JwtValidator::new(config.auth.url);

    let address = format!("{}:{}", config.application.host, config.application.port);

    let listener = TcpListener::bind(address)?;

    run(listener, client, jwt_validator)?.await
}