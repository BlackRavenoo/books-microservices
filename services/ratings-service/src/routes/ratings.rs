use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::schema::{BookRatingSchema, BulkGetSchema, GetListSchema, GetSchema, PaginationSchema, RateSchema, RatingSchema};

pub async fn get_rating(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    json: web::Json<GetSchema>
) -> impl Responder {
    let rating = if let Some(user_id) = json.user_id {
        sqlx::query_as!(
            RatingSchema,
            r#"
            SELECT avg_rating::REAL as "avg!",
                (SELECT rating FROM ratings WHERE book_id = $1 AND user_id = $2) as "user!"
            FROM book_rating_stats
            WHERE book_id = $1
            "#,
            path.into_inner(),
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
            path.into_inner()
        )
        .fetch_one(pool.get_ref())
        .await
    };

    match rating {
        Ok(rating) => HttpResponse::Ok().json(rating),
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

pub async fn get_list(
    pool: web::Data<PgPool>,
    schema: web::Json<GetListSchema>
) -> impl Responder {
    let schema = schema.into_inner();
    let page = schema.page.unwrap_or(1) - 1;
    let page_size = schema.page_size.unwrap_or(50);

    let ratings_list = sqlx::query_as!(
        BookRatingSchema,
        r#"
        SELECT book_id, avg_rating::REAL as "avg_rating!"
        FROM book_rating_stats
        ORDER BY avg_rating DESC
        OFFSET $1 LIMIT $2
        "#,
        (page * page_size) as i64,
        page_size as i64
    )
    .fetch_all(pool.as_ref())
    .await;
    
    let total_items = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM book_rating_stats"
    )
    .fetch_one(pool.as_ref())
    .await;

    match (ratings_list, total_items) {
        (Ok(ratings), Ok(Some(count))) => {
            let total_items = count as u64;
            let max_page = ((total_items as f64) / (page_size as f64)).ceil() as u64;
            
            HttpResponse::Ok().json(PaginationSchema{
                max_page,
                total_items,
                items: ratings,
            })
        },
        (Err(e), _) | (_, Err(e)) => {
            tracing::error!("Failed to get ratings or pagination data: {}", e);
            HttpResponse::InternalServerError().finish()
        },
        (_, Ok(None)) => {
            tracing::error!("COUNT query returned NULL");
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