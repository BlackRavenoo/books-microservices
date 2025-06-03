use actix_web::web;
use ratings::{bulk_get, get_rating, rate};

pub mod ratings;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ratings")
            .route("/bulk_get", web::post().to(bulk_get))
            .route("/rate", web::post().to(rate))
            .route("/{id}", web::post().to(get_rating))
    );
}