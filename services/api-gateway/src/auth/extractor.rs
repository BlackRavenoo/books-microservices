use std::future::{ready, Ready};

use actix_web::{FromRequest, HttpMessage, HttpRequest};

use super::jwt::Claims;

pub struct UserId(pub Option<i32>);

impl FromRequest for UserId {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        match req.extensions().get::<Claims>() {
            Some(claims) => {
                let user_id = match claims.sub.parse() {
                    Ok(id) => Some(id),
                    Err(e) => {
                        tracing::error!("Failed to parse id from sub field: {:?}", e);
                        None
                    },
                };
                ready(Ok(UserId(user_id)))
            },
            None => ready(Ok(UserId(None))),
        }
    }
}