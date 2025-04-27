use std::net::TcpListener;

use actix_session::storage::RedisSessionStore;
use actix_web::{cookie::Key, dev::Server, web, App, HttpResponse, HttpServer};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::{auth::{jwt::JwtService, token_store::TokenStore}, config::Settings, routes::oauth::configure_routes, utils::session_middleware};


pub fn run(
    listener: TcpListener,
    jwt_service: JwtService,
    db_pool: PgPool,
    store: TokenStore,
    redis_store: RedisSessionStore,
    config: Settings,
) -> Result<Server, std::io::Error> {
    let jwt_service = web::Data::new(jwt_service);
    let db_pool = web::Data::new(db_pool);
    let store = web::Data::new(store);

    let secret_key = if let Some(key) =  config.session.secret_key {
        Key::from(key.expose_secret().as_bytes())
    } else {
        Key::generate()
    };

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(session_middleware(redis_store.clone(), secret_key.clone()))
            .app_data(jwt_service.clone())
            .app_data(db_pool.clone())
            .app_data(store.clone())
            .route("/health", web::to(HttpResponse::Ok))
            .configure(configure_routes)
    })
    .listen(listener)?
    .run();

    Ok(server)
}