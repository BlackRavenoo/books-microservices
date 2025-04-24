use actix_web::web;
use crud::{create_book, get_book, get_books, search_books, update_book};

pub mod crud;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("v1")
            .route("/list", web::get().to(get_books))
            .route("/books/{id}", web::get().to(get_book))
            .route("/books", web::post().to(create_book))
            .route("/books/{id}", web::put().to(update_book))
            .route("/search", web::get().to(search_books))
    );
}