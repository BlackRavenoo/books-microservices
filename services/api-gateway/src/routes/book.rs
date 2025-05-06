use actix_web::{dev::PeerAddr, web, HttpRequest, HttpResponse, Responder, ResponseError};

use crate::{client::ServiceClient, schema::{BooksListQuery, SearchQuery}};


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

pub async fn update_book(
    client: web::Data<ServiceClient>,
    req: HttpRequest,
    payload: web::Payload,
    peer_addr: Option<PeerAddr>
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: Check if user is admin
    client.update_book(req, payload, peer_addr).await
}

pub async fn search_book(
    client: web::Data<ServiceClient>,
    q: web::Query<SearchQuery>
) -> impl Responder {
    match client.search_book(q.into_inner()).await {
        Ok(books) => HttpResponse::Ok().json(books),
        Err(e) => e.error_response(),
    }
}