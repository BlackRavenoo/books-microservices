use actix_web::web;
use crud::{create_author, create_book, get_author, get_book, get_books, get_constants, update_author, update_book};
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
            .route("/authors/{id}", web::get().to(get_author))
            .route("/authors/{id}", web::put().to(update_author))
            .route("/authors", web::post().to(create_author))
    );
}