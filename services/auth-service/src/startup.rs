use std::net::TcpListener;

use actix_files::Files;
use actix_session::storage::RedisSessionStore;
use actix_web::{cookie::Key, dev::Server, http, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use secrecy::ExposeSecret;
use tracing_actix_web::TracingLogger;

use crate::{auth::{client_store::ClientStore, code_store::CodeStore, jwt::JwtService, token_store::TokenStore}, config::Settings, routes::{auth, jwks, oauth}, services::user::UserService, utils::session_middleware};

pub fn run(
    listener: TcpListener,
    jwt_service: JwtService,
    token_store: TokenStore,
    code_store: CodeStore,
    user_service: UserService,
    client_store: ClientStore,
    redis_store: RedisSessionStore,
    config: Settings,
) -> Result<Server, std::io::Error> {
    let jwt_service = web::Data::new(jwt_service);
    let token_store = web::Data::new(token_store);
    let code_store = web::Data::new(code_store);
    let client_store = web::Data::new(client_store);
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
            .wrap(
                Cors::default()
                .allowed_origin_fn(|origin, _req_head| {
                    origin.as_bytes().starts_with(b"http://localhost") ||
                    origin.as_bytes().starts_with(b"http://127.0.0.1")
                })
                .allowed_methods(["GET", "POST", "OPTIONS", "DELETE", "PUT"])
                .allowed_headers([http::header::AUTHORIZATION, http::header::CONTENT_TYPE])
                .supports_credentials()
            )
            .app_data(jwt_service.clone())
            .app_data(token_store.clone())
            .app_data(code_store.clone())
            .app_data(user_service.clone())
            .app_data(client_store.clone())
            .route("/health", web::to(HttpResponse::Ok))
            .configure(auth::configure_routes)
            .configure(oauth::configure_routes)
            .configure(jwks::configure_routes)
            .service(Files::new("/public", "./public"))
            .route(
                "/",
                web::get().to(|| async { 
                    HttpResponse::Ok()
                        .content_type("text/html")
                        .body(include_str!("../public/index.html"))
                })
            )
    })
    .listen(listener)?
    .run();

    Ok(server)
}