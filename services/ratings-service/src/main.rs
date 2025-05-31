use ratings_service::config::get_config;
use sqlx::postgres::PgPoolOptions;
use telemetry::{get_subscriber, init_subscriber};
use std::net::TcpListener;

use ratings_service::startup::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("ratings-service".into(), "info".into(), std::io::stdout);
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

    run(listener, connection_pool)?.await
}