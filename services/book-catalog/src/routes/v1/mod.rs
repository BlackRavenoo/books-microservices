use actix_web::web;
use crud::{get_book, get_books};

pub mod crud;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("v1")
            .route("/list", web::get().to(get_books))
            .route("/book", web::get().to(get_book))
    );
}