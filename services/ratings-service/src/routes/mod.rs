use actix_web::web;
use ratings::{bulk_get, get_list, get_rating, rate};

pub mod ratings;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ratings")
            .route("/bulk_get", web::post().to(bulk_get))
            .route("/{id}", web::post().to(get_rating))
            .route("", web::get().to(get_list))
            .route("/rate", web::post().to(rate))
    );
}