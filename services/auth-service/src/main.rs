use auth_service::{auth::{jwt::JwtService, token_store::TokenStore}, config::get_config};
use sqlx::postgres::PgPoolOptions;
use telemetry::{get_subscriber, init_subscriber};
use std::net::TcpListener;

use auth_service::startup::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("auth-service".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_config().unwrap();

    let connection_pool = PgPoolOptions::new()
        .connect_lazy_with(config.database.with_db());

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    let address = format!("{}:{}", config.application.host, config.application.port);

    let listener = TcpListener::bind(address)?;

    let jwt_service = JwtService::new(&config.auth)
        .unwrap();

    let redis_manager = RedisConnectionManager::new(config.redis.url.clone())
        .expect("Failed to create Redis manager");
    let redis_pool = Arc::new(
        Pool::builder()
            .build(redis_manager)
            .await
            .expect("Failed to build Redis pool"),
    );

    let redis_store = RedisSessionStore::new("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect to Redis");

    let store = TokenStore::new(redis_pool);

    run(listener, jwt_service, connection_pool, store, redis_store, config)?.await
}