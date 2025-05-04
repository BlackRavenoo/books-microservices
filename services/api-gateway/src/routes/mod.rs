use actix_web::web;
use book::{get_book, get_books};

pub mod book;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/books", web::get().to(get_books))
        .route("/book/{id}", web::get().to(get_book));
}