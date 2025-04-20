use std::net::TcpListener;

use actix_web::{dev::Server, web::{self, Data}, App, HttpResponse, HttpServer};
use tracing_actix_web::TracingLogger;

use crate::client::ServiceClient;

pub fn run(
    listener: TcpListener,
    client: ServiceClient
) -> Result<Server, std::io::Error> {
    let client = Data::new(client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(client.clone())
            .route("/health", web::to(HttpResponse::Ok))
    })
    .listen(listener)?
    .run();

    Ok(server)
}