use actix_web::{web, HttpResponse, Responder, ResponseError};
use serde_qs::actix::QsQuery;

use crate::{client::ServiceClient, schema::{Author, BookSchema, GetListSchema, SearchQuery}};

pub async fn get_books(
    client: web::Data<ServiceClient>,
    query: QsQuery<GetListSchema>,
) -> impl Responder {
    match client.get_books_list(&query).await {
        Ok(books) => HttpResponse::Ok().json(books),
        Err(e) => e.error_response()
    }
}

pub async fn get_book(
    client: web::Data<ServiceClient>,
    id: web::Path<u64>
) -> impl Responder {
    match client.get_book(id.into_inner()).await {
        Ok(book) => HttpResponse::Ok().json(book),
        Err(e) => e.error_response()
    }
}

pub async fn search_book(
    client: web::Data<ServiceClient>,
    q: web::Query<SearchQuery>
) -> impl Responder {
    match client.search::<BookSchema>(q.into_inner(), "books").await {
        Ok(books) => HttpResponse::Ok().json(books),
        Err(e) => e.error_response(),
    }
}

pub async fn search_authors(
    client: web::Data<ServiceClient>,
    q: web::Query<SearchQuery>
) -> impl Responder {
    match client.search::<Author>(q.into_inner(), "authors").await {
        Ok(books) => HttpResponse::Ok().json(books),
        Err(e) => e.error_response(),
    }
}

pub async fn get_constants(
    client: web::Data<ServiceClient>,
) -> impl Responder {
    match client.get_constants().await {
        Ok(consts) => HttpResponse::Ok().json(consts),
        Err(e) => e.error_response()
    }
}