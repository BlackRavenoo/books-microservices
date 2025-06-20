use std::net::TcpListener;

use actix_web::{dev::Server, web::{self, Data}, App, HttpResponse, HttpServer};
use tracing_actix_web::TracingLogger;

use crate::{auth::jwt::JwtValidator, client::ServiceClient, routes::configure_routes};

pub fn run(
    listener: TcpListener,
    client: ServiceClient,
    jwt_validator: JwtValidator
) -> Result<Server, std::io::Error> {
    let client = Data::new(client);
    let validator = Data::new(jwt_validator);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(client.clone())
            .app_data(validator.clone())
            .route("/health", web::to(HttpResponse::Ok))
            .configure(configure_routes)
    })
    .listen(listener)?
    .run();

    Ok(server)
}