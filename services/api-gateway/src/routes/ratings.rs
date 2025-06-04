use actix_web::{web, HttpResponse, Responder, ResponseError};

use crate::{auth::extractor::UserId, client::ServiceClient, schema::RateInputSchema};

pub async fn rate(
    client: web::Data<ServiceClient>,
    schema: web::Json<RateInputSchema>,
    user_id: UserId
) -> impl Responder {
    let user_id = match user_id.0 {
        Some(id) => id,
        None => {
            tracing::error!("User ID is none");
            return HttpResponse::BadRequest().finish();
        },
    };

    match client.rate(&schema, user_id).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => e.error_response()
    }
}