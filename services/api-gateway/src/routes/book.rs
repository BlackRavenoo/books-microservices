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