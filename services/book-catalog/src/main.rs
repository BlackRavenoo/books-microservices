use std::{net::TcpListener, time::Duration};

use bb8_redis::{RedisConnectionManager, bb8::Pool};
use book_catalog::{
    config::get_config, migration::Migrator, search::ElasticsearchClient, startup::run, storage::s3::S3StorageBackend
};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("book-catalog".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_config().expect("Failed to read config");
    
    let db = Database::connect(config.database.get_options()).await.unwrap();

    let search = ElasticsearchClient::new(&config.search.url, &config.search.books_index_name, &config.search.authors_index_name);

    search.create_books_index().await.unwrap();
    search.create_authors_index().await.unwrap();

    Migrator::up(&db, None).await
        .expect("Failed to migrate the database");

    let address = format!("{}:{}", config.application.host, config.application.port);

    let listener = TcpListener::bind(address)?;

    let redis_manager = RedisConnectionManager::new(config.cache.url.clone())
        .expect("Failed to create Redis manager");
    let redis_pool = Pool::builder()
            .connection_timeout(Duration::from_millis(100))
            .build(redis_manager)
            .await
            .expect("Failed to build Redis pool");

    let storage = S3StorageBackend::new(config.s3);

    run(listener, db, search, redis_pool, storage)?.await
}
