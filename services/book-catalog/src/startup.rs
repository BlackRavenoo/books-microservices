use std::net::TcpListener;

use actix_web::{dev::Server, web::{self, Data}, App, HttpResponse, HttpServer};
use bb8_redis::{bb8::Pool, RedisConnectionManager};
use cache::{cache::HybridCache, serializer::bincode::BincodeSerializer};
use metrics_exporter_prometheus::PrometheusBuilder;
use sea_orm::DatabaseConnection;
use tracing_actix_web::TracingLogger;

use crate::{routes::{metrics::get_metrics, v1}, schema::{BookFullSchema, ConstantsSchema}, search::elasticsearch::ElasticsearchClient, storage::s3::S3StorageBackend};

pub fn run(
    listener: TcpListener,
    db: DatabaseConnection,
    search: ElasticsearchClient,
    redis_pool: Pool<RedisConnectionManager>,
    storage: S3StorageBackend
) -> Result<Server, std::io::Error> {
    let db = Data::new(db);
    let search = Data::new(search);
    let storage = Data::new(storage);

    let constants_cache = Data::new(HybridCache::<String, ConstantsSchema, BincodeSerializer<_>>::new(
        "constants".to_string(),
        redis_pool.clone(),
        1,
        BincodeSerializer::default()
    ));
    
    let book_full_cache = Data::new(HybridCache::<String, BookFullSchema, BincodeSerializer<_>>::new(
        "book-full".to_string(),
        redis_pool,
        50000,
        BincodeSerializer::default()
    ));

    let builder = PrometheusBuilder::new();

    let handle = builder
        .add_global_label("service", "book-catalog")
        .install_recorder()
        .expect("Failed to install metrics recorder");

    let handle = Data::new(handle);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(db.clone())
            .app_data(search.clone())
            .app_data(constants_cache.clone())
            .app_data(book_full_cache.clone())
            .app_data(storage.clone())
            .route("/health", web::to(HttpResponse::Ok))
            .service(
                web::scope("/api")
                .configure(v1::config)
            )
            // Metrics
            .app_data(handle.clone())
            .route("/metrics", web::get().to(get_metrics))
    })
    .listen(listener)?
    .run();

    Ok(server)
}