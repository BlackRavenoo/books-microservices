use std::net::TcpListener;

use actix_web::{dev::Server, web::{self, Data}, App, HttpResponse, HttpServer};
use bb8_redis::{bb8::Pool, RedisConnectionManager};
use cache::{actix::CacheMiddleware, cache::HybridCache, serializer::bincode::BincodeSerializer};
use tracing_actix_web::TracingLogger;

use crate::{auth::jwt::JwtValidator, client::ServiceClient, routes::configure_routes};

pub fn run(
    listener: TcpListener,
    client: ServiceClient,
    jwt_validator: JwtValidator,
    redis_pool: Pool<RedisConnectionManager>,
) -> Result<Server, std::io::Error> {
    let client = Data::new(client);
    let validator = Data::new(jwt_validator);

    let cache = HybridCache::new("api-gateway".to_string(), redis_pool, 2000, BincodeSerializer::default());

    let server = HttpServer::new(move || {
        let cache_middleware = CacheMiddleware::new(cache.clone())
            .cache_condition(|ctx| {
                ctx.method == "GET" && ctx.path != "/books"
            });
        
        App::new()
            .wrap(TracingLogger::default())
            .wrap(cache_middleware)
            .app_data(client.clone())
            .app_data(validator.clone())
            .route("/health", web::to(HttpResponse::Ok))
            .configure(configure_routes)
    })
    .listen(listener)?
    .run();

    Ok(server)
}