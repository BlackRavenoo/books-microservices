use actix_web::{web, HttpResponse, Responder, ResponseError};

use crate::{client::ServiceClient, schema::BooksListQuery};


// TODO: Bool "with_rating" field
pub async fn get_books(
    client: web::Data<ServiceClient>,
    query: web::Query<BooksListQuery>,
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