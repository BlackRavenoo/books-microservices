use std::{net::TcpListener, sync::Arc};

use actix_web::{dev::Server, web::{self, Data}, App, HttpResponse, HttpServer};
use bb8_redis::{bb8::Pool, RedisConnectionManager};
use cache::cache::HybridCache;
use sea_orm::DatabaseConnection;
use tracing_actix_web::TracingLogger;

use crate::{routes::v1, schema::{BookFullSchema, ConstantsSchema}, search::elasticsearch::ElasticsearchClient};

pub fn run(
    listener: TcpListener,
    db: DatabaseConnection,
    search: ElasticsearchClient,
    redis_pool: Arc<Pool<RedisConnectionManager>>
) -> Result<Server, std::io::Error> {
    let db = Data::new(db);
    let search = Data::new(search);

    let constants_cache = Data::new(HybridCache::<String, ConstantsSchema>::new(
        "constants".to_string(),
        Arc::clone(&redis_pool),
        1,
    ));
    
    let book_full_cache = Data::new(HybridCache::<String, BookFullSchema>::new(
        "book-full".to_string(),
        Arc::clone(&redis_pool),
        50000,
    ));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(db.clone())
            .app_data(search.clone())
            .app_data(constants_cache.clone())
            .app_data(book_full_cache.clone())
            .route("/health", web::to(HttpResponse::Ok))
            .service(
                web::scope("/api")
                    .configure(v1::config)
            )
    })
    .listen(listener)?
    .run();

    Ok(server)
}