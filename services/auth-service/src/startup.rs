use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::{auth::jwt::JwtService, routes::jwt::configure_routes};


pub fn run(
    listener: TcpListener,
    jwt_service: JwtService,
    db_pool: PgPool
) -> Result<Server, std::io::Error> {
    let jwt_service = web::Data::new(jwt_service);
    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(jwt_service.clone())
            .app_data(db_pool.clone())
            .route("/health", web::to(HttpResponse::Ok))
            .configure(configure_routes)
    })
    .listen(listener)?
    .run();

    Ok(server)
}