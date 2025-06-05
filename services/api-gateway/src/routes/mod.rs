use actix_web::web;
use author::get_author;
use book::{get_book, get_books, search_authors, search_book, get_constants};
use chapter::{get_chapter, get_chapters};
use entity::{create_entity, delete_entity, update_entity};

use crate::{auth::middleware::JwtMiddleware, routes::ratings::rate};

pub mod book;
pub mod author;
pub mod entity;
pub mod chapter;
pub mod ratings;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            web::scope("/books")
            .route("", web::get().to(get_books))
            .route("/{id}/chapter", web::get().to(get_chapter))
            .route("/{id}/chapters", web::get().to(get_chapters))
            .route("/{id}", web::get().to(get_book)).wrap(JwtMiddleware::optional())
            .service(
                web::scope("")
                    .wrap(JwtMiddleware::admin_only())
                    .route("", web::post().to(create_entity))
                    .route("/{id}/chapter", web::post().to(create_entity))
                    .route("/{id}/chapter", web::put().to(update_entity))
                    .route("/{id}/chapter", web::delete().to(delete_entity))
                    .route("/{id}", web::put().to(update_entity))
            )
        )
        .service(
            web::scope("/authors")
            .route("/{id}", web::get().to(get_author))
            .service(
                web::scope("")
                    .wrap(JwtMiddleware::admin_only())
                    .route("", web::post().to(create_entity))
                    .route("/{id}", web::delete().to(delete_entity))
                    .route("/{id}", web::put().to(update_entity))
                
            )
        )
        .service(
            web::scope("/search")
                .route("/books", web::get().to(search_book))
                .route("/authors", web::get().to(search_authors))
        )
        .service(
            web::scope("/ratings")
                .wrap(JwtMiddleware::default())
                .route("/rate", web::post().to(rate))
        )
        .route("/constants", web::get().to(get_constants));
}