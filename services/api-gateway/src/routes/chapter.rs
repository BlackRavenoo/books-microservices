use actix_web::{web, HttpResponse, Responder, ResponseError};

use crate::{client::ServiceClient, schema::InputChapterSchema};

pub async fn get_chapter(
    client: web::Data<ServiceClient>,
    path: web::Path<u64>,
    query: web::Query<InputChapterSchema>
) -> impl Responder {
    match client.get_chapter(path.into_inner(), query.into_inner()).await {
        Ok(chapter) => HttpResponse::Ok().json(chapter),
        Err(e) => e.error_response()
    }
}

pub async fn get_chapters(
    client: web::Data<ServiceClient>,
    path: web::Path<u64>
) -> impl Responder {
    match client.get_chapters_list(path.into_inner()).await {
        Ok(chapters) => HttpResponse::Ok().json(chapters),
        Err(e) => e.error_response(),
    }
}