use std::net::TcpListener;

use actix_web::{dev::Server, web::{self, Data}, App, HttpResponse, HttpServer};
use bb8_redis::{bb8::Pool, RedisConnectionManager};
use cache::{cache::HybridCache, serializer::bincode::BincodeSerializer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::{routes::configure_routes, schema::RatingSchema};

pub fn run(
    listener: TcpListener,
    pool: PgPool,
    redis_pool: Pool<RedisConnectionManager>,
) -> Result<Server, std::io::Error> {
    let pool = web::Data::new(pool);

    let cache = Data::new(HybridCache::<String, RatingSchema, BincodeSerializer<_>>::new(
        "ratings".to_string(),
        redis_pool,
        10000,
        BincodeSerializer::default()
    ));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(pool.clone())
            .app_data(cache.clone())
            .route("/health", web::to(HttpResponse::Ok))
            .configure(configure_routes)
    })
    .listen(listener)?
    .run();

    Ok(server)
}