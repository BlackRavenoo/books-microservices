use actix_web::{dev::PeerAddr, web, HttpRequest, HttpResponse, Responder, ResponseError};

use crate::client::ServiceClient;

pub async fn update_entity(
    client: web::Data<ServiceClient>,
    req: HttpRequest,
    payload: web::Payload,
    peer_addr: Option<PeerAddr>
) -> Result<HttpResponse, actix_web::Error> {
    client.update_entity(req, payload, peer_addr).await
}

pub async fn create_entity(
    client: web::Data<ServiceClient>,
    req: HttpRequest,
    payload: web::Payload,
    peer_addr: Option<PeerAddr>
) -> Result<HttpResponse, actix_web::Error> {
    client.create_entity(req, payload, peer_addr).await
}

pub async fn delete_entity(
    client: web::Data<ServiceClient>,
    req: HttpRequest
) -> impl Responder {
    match client.delete_entity(req).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to delete entity");
            return e.error_response()
        },
    }
}