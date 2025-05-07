use actix_web::{web, HttpResponse, Responder};

use crate::{schema::SearchSchema, search::ElasticsearchClient};

pub async fn search_books(search: web::Data<ElasticsearchClient>, query: web::Query<SearchSchema>) -> impl Responder {
    match search.search_books(&query.q).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            tracing::error!("Failed to search book: {:?}", e);
            HttpResponse::BadRequest().finish()
        },
    }
}

pub async fn search_authors(search: web::Data<ElasticsearchClient>, query: web::Query<SearchSchema>) -> impl Responder {
    match search.search_authors(&query.q).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            tracing::error!("Failed to search author: {:?}", e);
            HttpResponse::BadRequest().finish()
        },
    }
}