use actix_web::web;
use book::{get_book, get_books, search_book, update_book};

pub mod book;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/books", web::get().to(get_books))
        .route("/books/{id}", web::get().to(get_book))
        .route("/search", web::get().to(search_book))
        .route("/books/{id}", web::put().to(update_book));
}