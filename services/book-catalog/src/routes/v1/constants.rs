use actix_web::{web, HttpResponse, Responder};
use cache::{cache::HybridCache, expiry::Expiration, serializer::bincode::BincodeSerializer};
use sea_orm::{DatabaseConnection, EntityTrait, Iterable};

use crate::{
    entity::{
        book::BookStatus, genre, tag
    }, schema::{ConstantsSchema, Genre, Tag}
};

pub async fn get_constants(
    db: web::Data<DatabaseConnection>,
    cache: web::Data<HybridCache<String, ConstantsSchema, BincodeSerializer<ConstantsSchema>>>
) -> impl Responder {
    match cache.get(&String::new(), Expiration::Minutes(10)).await {
        Ok(Some(consts)) => return HttpResponse::Ok().json(consts),
        Ok(None) => (),
        Err(e) => tracing::error!("Failed to get constants from cache: {:?}", e),
    };

    let tags = match tag::Entity::find()
        .into_partial_model::<Tag>()
        .all(db.as_ref())
        .await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to fetch tags from db: {:?}", e);
                return HttpResponse::InternalServerError().finish()
            },
        };

    let genres = match genre::Entity::find()
        .into_partial_model::<Genre>()
        .all(db.as_ref())
        .await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to fetch genres from db: {:?}", e);
                return HttpResponse::InternalServerError().finish()
            },
        };

    let constants = ConstantsSchema {
        tags,
        genres,
        status: BookStatus::iter().collect()
    };

    if let Err(e) = cache.set(String::new(), constants.clone(), Expiration::Minutes(10)).await {
        tracing::error!("Failed to insert constants into cache: {:?}", e);
    };

    HttpResponse::Ok().json(constants)
}