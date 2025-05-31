use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::schema::{BooksRatingSchema, BulkGetSchema};

pub async fn bulk_get(
    pool: web::Data<PgPool>,
    schema: web::Json<BulkGetSchema>
) -> impl Responder {
    let ids = schema.into_inner().ids;

    if ids.is_empty() {
        return HttpResponse::BadGateway().finish();
    }

    let ratings = sqlx::query_as!(
        BooksRatingSchema,
        r#"
        SELECT book_id, avg_rating::TEXT as "avg_rating!"
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

