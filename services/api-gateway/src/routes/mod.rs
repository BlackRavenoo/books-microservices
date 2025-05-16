use actix_web::web;
use author::get_author;
use book::{get_book, get_books, search_authors, search_book, update_entity, create_entity, get_constants};

pub mod book;
pub mod author;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/books", web::get().to(get_books))
        .route("/books/{id}", web::get().to(get_book))
        .route("/search/books", web::get().to(search_book))
        .route("/search/authors", web::get().to(search_authors))
        .route("/books/{id}", web::put().to(update_entity))
        .route("/books", web::post().to(create_entity))
        .route("/authors/{id}", web::put().to(update_entity))
        .route("/authors", web::post().to(create_entity))
        .route("/constants", web::get().to(get_constants))
        .route("/authors/{id}", web::get().to(get_author));
}