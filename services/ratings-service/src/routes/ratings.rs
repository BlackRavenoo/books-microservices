use actix_web::{web, HttpResponse, Responder};
use cache::{cache::HybridCache, expiry::Expiration, serializer::bincode::BincodeSerializer};
use sqlx::PgPool;

use crate::schema::{BookRatingSchema, BulkGetSchema, GetSchema, RateSchema, RatingSchema};

pub async fn get_rating(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    json: web::Json<GetSchema>,
    cache: web::Data<HybridCache::<String, RatingSchema, BincodeSerializer<RatingSchema>>>
) -> impl Responder {
    let path = path.into_inner();
    let key = format!("{}_{}", path, json.user_id.unwrap_or(-1));
    match cache.get(
        &key,
        Expiration::Minutes(10)
    ).await {
        Ok(Some(rating)) => return HttpResponse::Ok().json(rating),
        Ok(None) => (),
        Err(e) => tracing::error!("Failed to get rating from cache: {:?}", e),
    };

    let rating = if let Some(user_id) = json.user_id {
        sqlx::query_as!(
            RatingSchema,
            r#"
            SELECT avg_rating::REAL as "avg!",
                (SELECT rating FROM ratings WHERE book_id = $1 AND user_id = $2) as "user!"
            FROM book_rating_stats
            WHERE book_id = $1
            "#,
            path,
            user_id
        )
        .fetch_one(pool.get_ref())
        .await
    } else {
        sqlx::query_as!(
            RatingSchema,
            r#"
            SELECT avg_rating::REAL as "avg!", NULL::SMALLINT as "user"
            FROM book_rating_stats
            WHERE book_id = $1
            "#,
            path
        )
        .fetch_one(pool.get_ref())
        .await
    };

    match rating {
        Ok(rating) => {
            if let Err(e) = cache.set(
                key,
                rating.clone(),
                Expiration::Minutes(10)
            ).await {
                tracing::error!("Failed to insert book into cache: {:?}", e)
            }
            HttpResponse::Ok().json(rating)
        },
        Err(e) => {
            tracing::error!("Failed to get rating: {:?}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn bulk_get(
    pool: web::Data<PgPool>,
    schema: web::Json<BulkGetSchema>
) -> impl Responder {
    let ids = schema.into_inner().ids;

    if ids.is_empty() {
        return HttpResponse::BadGateway().finish();
    }

    let ratings = sqlx::query_as!(
        BookRatingSchema,
        r#"
        SELECT book_id, avg_rating::REAL as "avg_rating!"
        FROM book_rating_stats
        WHERE book_id = ANY($1)
        "#,
        &ids
    )
    .fetch_all(pool.get_ref())
    .await;

    match ratings {
        Ok(ratings) => HttpResponse::Ok().json(ratings),
        Err(e) => {
            tracing::error!("Failed to bulk get ratings: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn rate(
    pool: web::Data<PgPool>,
    schema: web::Json<RateSchema>
) -> impl Responder {
    let schema = schema.into_inner();

    match schema.score {
        0 => {
            let result = sqlx::query!(
                r#"
                DELETE FROM ratings
                WHERE user_id = $1 AND book_id = $2
                "#,
                schema.user_id,
                schema.item_id
            )
            .execute(pool.as_ref())
            .await;

            match result {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => {
                    tracing::error!("Failed to delete rating: {:?}", e);
                    HttpResponse::InternalServerError().finish()
                },
            }
        },
        1..=10 => {
            let result = sqlx::query!(
                r#"
                INSERT INTO ratings(user_id, book_id, rating)
                VALUES ($1, $2, $3)
                ON CONFLICT (user_id, book_id)
                DO UPDATE SET rating = EXCLUDED.rating
                "#,
                schema.user_id,
                schema.item_id,
                schema.score
            )
            .execute(pool.as_ref())
            .await;

            match result {
                Ok(_) => HttpResponse::Created().finish(),
                Err(e) => {
                    tracing::error!("Failed to insert rating: {:?}", e);
                    HttpResponse::InternalServerError().finish()
                },
            }
        },
        _ => {
            HttpResponse::BadRequest().finish()
        }
    }
}