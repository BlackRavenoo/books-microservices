use actix_session::{config::PersistentSession, storage::RedisSessionStore, SessionMiddleware};
use actix_web::cookie::SameSite;
use time::Duration;

pub fn session_middleware(redis_store: RedisSessionStore, secret_key: actix_web::cookie::Key) -> SessionMiddleware<RedisSessionStore> {
    SessionMiddleware::builder(redis_store, secret_key)
        .cookie_name("auth_session".to_owned())
        .cookie_secure(true)
        .cookie_http_only(true)
        .cookie_same_site(SameSite::Strict)
        .session_lifecycle(
            PersistentSession::default().session_ttl(Duration::hours(24))
        )
        .build()
}