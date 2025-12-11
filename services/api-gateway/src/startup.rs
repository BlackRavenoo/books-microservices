use std::net::TcpListener;

use actix_web::{dev::Server, web::{self, Data}, App, HttpResponse, HttpServer};
use bb8_redis::{bb8::Pool, RedisConnectionManager};
use cache::{actix::CacheMiddleware, cache::HybridCache, serializer::bincode::BincodeSerializer};
use metrics_exporter_prometheus::PrometheusBuilder;
use tracing_actix_web::TracingLogger;

use crate::{auth::jwt::JwtValidator, client::ServiceClient, routes::{configure_routes, metrics::get_metrics}};

pub fn run(
    listener: TcpListener,
    client: ServiceClient,
    jwt_validator: JwtValidator,
    redis_pool: Pool<RedisConnectionManager>,
) -> Result<Server, std::io::Error> {
    let client = Data::new(client);
    let validator = Data::new(jwt_validator);

    let cache = HybridCache::new("api-gateway".to_string(), redis_pool, 2000, BincodeSerializer::default());

    let builder = PrometheusBuilder::new();

    let handle = builder
        .add_global_label("service", "api-gateway")
        .install_recorder()
        .expect("Failed to install metrics recorder");

    let handle = Data::new(handle);

    let server = HttpServer::new(move || {
        let cache_middleware = CacheMiddleware::new(cache.clone())
            .cache_condition(|ctx| {
                ctx.method == "GET" && ctx.path != "/books"
            });
        
        App::new()
            .wrap(cache_middleware)
            .wrap(TracingLogger::default())
            .app_data(client.clone())
            .app_data(validator.clone())
            .app_data(handle.clone())
            .route("/health", web::to(HttpResponse::Ok))
            .route("/metrics", web::get().to(get_metrics))
            .configure(configure_routes)
    })
    .listen(listener)?
    .run();

    Ok(server)
}