use actix_web::web;
use crud::{create_book, get_book, get_books, get_constants, update_book};
use search::{search_authors, search_books};

mod crud;
mod search;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("v1")
            .route("/books", web::get().to(get_books))
            .route("/books/{id}", web::get().to(get_book))
            .route("/books", web::post().to(create_book))
            .route("/books/{id}", web::put().to(update_book))
            .route("/search/books", web::get().to(search_books))
            .route("/search/authors", web::get().to(search_authors))
            .route("/constants", web::get().to(get_constants))
    );
}