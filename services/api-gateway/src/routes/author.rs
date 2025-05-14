use actix_web::{web, HttpResponse, Responder, ResponseError};

use crate::client::ServiceClient;

pub async fn get_author(
    client: web::Data<ServiceClient>,
    id: web::Path<u64>,
) -> impl Responder {
    match client.get_author(id.into_inner()).await {
        Ok(author) => HttpResponse::Ok().json(author),
        Err(e) => e.error_response()
    }
}