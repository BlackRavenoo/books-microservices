use std::net::TcpListener;

use actix_web::{dev::Server, web::{self, Data}, App, HttpResponse, HttpServer};
use sea_orm::DatabaseConnection;
use tracing_actix_web::TracingLogger;

use crate::{config::Settings, routes::v1, search::elasticsearch::ElasticsearchClient};

pub fn run(
    listener: TcpListener,
    db: DatabaseConnection,
    search: ElasticsearchClient,
    settings: Settings
) -> Result<Server, std::io::Error> {
    let db = Data::new(db);
    let search = Data::new(search);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(db.clone())
            .app_data(search.clone())
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