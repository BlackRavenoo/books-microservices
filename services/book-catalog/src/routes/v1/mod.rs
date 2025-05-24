use actix_web::web;
use authors::{create_author, delete_author, get_author, update_author};
use books::{create_book, get_book, get_books, update_book};
use constants::get_constants;
use search::{search_authors, search_books};

mod constants;
mod search;
mod books;
mod authors;

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
            .route("/authors", web::delete().to(delete_author))
    );
}