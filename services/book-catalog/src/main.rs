use std::net::TcpListener;

use book_catalog::{
    config::get_config,
    migration::Migrator,
    search::ElasticsearchClient,
    startup::run,
};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("server".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_config().expect("Failed to read config");
    
    let db = Database::connect(config.database.get_options()).await.unwrap();

    let search = ElasticsearchClient::new(&config.search.url, &config.search.index_name);

    search.create_index().await.unwrap();

    Migrator::up(&db, None).await
        .expect("Failed to migrate the database");

    let address = format!("{}:{}", config.application.host, config.application.port);

    let listener = TcpListener::bind(address)?;

    run(listener, db, search, config)?.await
}
