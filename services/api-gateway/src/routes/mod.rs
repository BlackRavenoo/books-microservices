use actix_web::web;
use author::get_author;
use book::{get_book, get_books, search_authors, search_book, update_book, create_book, get_constants};

pub mod book;
pub mod author;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/books", web::get().to(get_books))
        .route("/books/{id}", web::get().to(get_book))
        .route("/search/books", web::get().to(search_book))
        .route("/search/authors", web::get().to(search_authors))
        .route("/books/{id}", web::put().to(update_book))
        .route("/books", web::post().to(create_book))
        .route("/constants", web::get().to(get_constants))
        .route("/authors/{id}", web::get().to(get_author));
}