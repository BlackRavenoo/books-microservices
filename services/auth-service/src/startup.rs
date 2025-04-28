use std::net::TcpListener;

use actix_session::storage::RedisSessionStore;
use actix_web::{cookie::Key, dev::Server, web, App, HttpResponse, HttpServer};
use secrecy::ExposeSecret;
use tracing_actix_web::TracingLogger;

use crate::{auth::{code_store::CodeStore, jwt::JwtService, token_store::TokenStore}, config::Settings, routes::oauth::configure_routes, services::user::UserService, utils::session_middleware};


pub fn run(
    listener: TcpListener,
    jwt_service: JwtService,
    token_store: TokenStore,
    code_store: CodeStore,
    user_service: UserService,
    redis_store: RedisSessionStore,
    config: Settings,
) -> Result<Server, std::io::Error> {
    let jwt_service = web::Data::new(jwt_service);
    let token_store = web::Data::new(token_store);
    let code_store = web::Data::new(code_store);
    let user_service = web::Data::new(user_service);

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
            .app_data(token_store.clone())
            .app_data(code_store.clone())
            .app_data(user_service.clone())
            .route("/health", web::to(HttpResponse::Ok))
            .configure(configure_routes)
    })
    .listen(listener)?
    .run();

    Ok(server)
}