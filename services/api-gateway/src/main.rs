use api_gateway::{auth::jwt::JwtValidator, client::ServiceClient};
use api_gateway::config::get_config;
use bb8_redis::bb8::Pool;
use bb8_redis::RedisConnectionManager;
use telemetry::{get_subscriber, init_subscriber};
use std::net::TcpListener;
use std::time::Duration;

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

    let redis_manager = RedisConnectionManager::new(config.cache.url.clone())
        .expect("Failed to create Redis manager");
    let redis_pool = Pool::builder()
            .connection_timeout(Duration::from_millis(100))
            .build(redis_manager)
            .await
            .expect("Failed to build Redis pool");

    run(listener, client, jwt_validator, redis_pool)?.await
}