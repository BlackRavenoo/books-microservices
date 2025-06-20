use actix_web::web;
use authors::{create_author, delete_author, get_author, update_author};
use books::{create_book, get_book, get_books, update_book, delete_book};
use chapters::{create_chapter, delete_chapter, get_chapter, get_chapters, update_chapter};
use constants::get_constants;
use search::{search_authors, search_books};

mod constants;
mod search;
mod books;
mod authors;
mod chapters;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("v1")
            .service(
                web::scope("/books")
                    .route("", web::get().to(get_books))
                    .route("", web::post().to(create_book))
                    .route("/{id}/chapter", web::post().to(create_chapter))
                    .route("/{id}/chapter", web::get().to(get_chapter))
                    .route("/{id}/chapters", web::get().to(get_chapters))
                    .route("/{id}/chapter", web::put().to(update_chapter))
                    .route("/{id}/chapter", web::delete().to(delete_chapter))
                    .route("/{id}", web::get().to(get_book))
                    .route("/{id}", web::put().to(update_book))
                    .route("/{id}", web::delete().to(delete_book))
            )
            .service(
                web::scope("/authors")
                    .route("", web::post().to(create_author))
                    .route("/{id}", web::delete().to(delete_author))
                    .route("/{id}", web::get().to(get_author))
                    .route("/{id}", web::put().to(update_author))
            )
            .service(
                web::scope("/search")
                    .route("/books", web::get().to(search_books))
                    .route("/authors", web::get().to(search_authors))
            )
            .route("/constants", web::get().to(get_constants))
    );
}