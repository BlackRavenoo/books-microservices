use std::{future::{ready, Future, Ready}, marker::PhantomData, pin::Pin, rc::Rc, sync::Arc, task::{Context, Poll}};

use actix_web::{body::{BodySize, BoxBody, EitherBody, MessageBody}, dev::{forward_ready, Response, Service, ServiceRequest, ServiceResponse, Transform}, http::{header::HeaderMap, StatusCode}, web::{Bytes, BytesMut}, Error, HttpMessage};
use bincode::{Decode, Encode};
use futures_util::{future::LocalBoxFuture, StreamExt};
use pin_project_lite::pin_project;

use crate::{cache::HybridCache, expiry::Expiration, serializer::bincode::BincodeSerializer};

pub struct RequestContext<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub query_string: &'a str,
    pub headers: &'a HeaderMap,
    pub body: &'a LazyBody
}

pub struct LazyBody {
    raw: Bytes,
    parsed: std::sync::OnceLock<serde_json::Value>,
}

impl LazyBody {
    pub fn new(data: BytesMut) -> Self {
        Self {
            raw: data.freeze(),
            parsed: std::sync::OnceLock::new(),
        }
    }
    
    pub fn as_json(&self) -> &serde_json::Value {
        self.parsed.get_or_init(|| {
            serde_json::from_slice(&self.raw).unwrap_or(serde_json::Value::Null)
        })
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.raw
    }
}

type CacheConditionFn = Arc<dyn Fn(&RequestContext) -> bool + Send + Sync>;

type KeyGenerator = Arc<dyn Fn(&RequestContext) -> String + Send + Sync>;

type Cache = HybridCache<String, CacheEntry, BincodeSerializer<CacheEntry>>;

pub struct CacheMiddleware {
    cache: Cache,
    config: CacheConfig,
}

#[derive(Clone)]
pub struct CacheConfig {
    ttl: Expiration,
    max_cache_size: usize,
    cache_condition: CacheConditionFn,
    key_gen_fn: KeyGenerator,
    requires_body: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl: Expiration::Minutes(60),
            max_cache_size: 1024 * 32,
            cache_condition: Arc::new(|_| true),
            key_gen_fn: Arc::new(generate_key_default),
            requires_body: false
        }
    }
}

fn generate_key_default(ctx: &RequestContext) -> String {
    format!(
        "{}_{}_{}",
        ctx.method,
        ctx.path,
        ctx.query_string
    )
}

impl CacheMiddleware {
    pub fn new(cache: Cache) -> Self {
        Self {
            cache,
            config: CacheConfig::default()
        }
    }

    pub fn with_config(cache: Cache, config: CacheConfig) -> Self {
        Self {
            cache,
            config
        }
    }

    pub fn ttl(mut self, ttl: Expiration) -> Self {
        self.config.ttl = ttl;
        self
    }

    pub fn max_cache_size(mut self, size: usize) -> Self {
        self.config.max_cache_size = size;
        self
    }

    pub fn cache_condition<F>(mut self, condition: F) -> Self 
    where
        F: Fn(&RequestContext) -> bool + Send + Sync + 'static,
    {
        self.config.cache_condition = Arc::new(condition);
        self
    }

    pub fn key_gen_fn<F>(mut self, key_fn: F) -> Self
    where
        F: Fn(&RequestContext) -> String + Send + Sync + 'static,
    {
        self.config.key_gen_fn = Arc::new(key_fn);
        self
    }

    pub fn requires_body(mut self, requires_body: bool) -> Self {
        self.config.requires_body = requires_body;
        self
    }
}

pub struct CacheMiddlewareService<S> {
    service: Rc<S>,
    cache: Cache,
    config: CacheConfig,
}

#[derive(Decode, Encode, Clone)]
pub struct CacheEntry {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>
}

impl<S, B> Transform<S, ServiceRequest> for CacheMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static + MessageBody
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Transform = CacheMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CacheMiddlewareService {
            service: Rc::new(service),
            cache: self.cache.clone(),
            config: self.config.clone(),
        }))
    }
}

pin_project! {
    struct CacheResponseFuture<S, B>
    where
        B: MessageBody,
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>
    {
        #[pin]
        fut: S::Future,
        should_cache: bool,
        cache_key: String,
        cache: Cache,
        ttl: Expiration,
        max_cache_size: usize,
        _marker: PhantomData<B>,
    }
}

impl<S, B> Future for CacheResponseFuture<S, B>
where
    B: MessageBody + 'static,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>
{
    type Output = Result<ServiceResponse<EitherBody<B, BoxBody>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let res = futures_util::ready!(this.fut.poll(cx))?;

        let status = res.status();
        
        if !(*this.should_cache && status.is_success()) {
            return Poll::Ready(Ok(res.map_body(|_, b| EitherBody::left(b))));
        }

        let headers = res.headers().clone();

        let cache_key = this.cache_key.clone();
        let ttl = *this.ttl;
        let max_size = *this.max_cache_size;
        let cache = this.cache.clone();

        let res = res.map_body(move |_, body| {
            let filtered_headers = headers
                .iter()
                .filter(|(name, _)| {
                    !matches!(
                        name.as_str().to_lowercase().as_str(),
                        "connection" | "transfer-encoding" | "content-length"
                    )
                })
                .map(|(name, value)| {
                    (
                        name.to_string(),
                        value.to_str().unwrap_or_default().to_string(),
                    )
                })
                .collect::<Vec<_>>();

            EitherBody::right(BoxBody::new(CacheableBody {
                body: body.boxed(),
                status: status.as_u16(),
                headers: filtered_headers,
                body_buffer: BytesMut::new(),
                cache_key,
                cache,
                ttl,
                max_size,
            }))
        });

        Poll::Ready(Ok(res))
    }
}

pin_project! {
    pub struct CacheableBody {
        #[pin]
        body: BoxBody,
        status: u16,
        headers: Vec<(String, String)>,
        body_buffer: BytesMut,
        cache_key: String,
        cache: Cache,
        ttl: Expiration,
        max_size: usize,
    }

    impl PinnedDrop for CacheableBody {
        fn drop(this: Pin<&mut Self>) {
            let this = this.project();

            let body = this.body_buffer.clone().freeze();

            if !body.is_empty() && body.len() <= *this.max_size {
                let status = *this.status;
                let headers = this.headers.clone();
                let cache = this.cache.clone();
                let ttl = *this.ttl;
                let key = this.cache_key.clone();

                actix_web::rt::spawn(async move {
                    let entry = CacheEntry {
                        status,
                        headers,
                        body: body.to_vec()
                    };

                    if let Err(e) = cache.set(key, entry, ttl).await {
                        tracing::error!("Failed to insert value in cache: {:?}", e);
                    };
                });
            }
        }
    }
}

impl MessageBody for CacheableBody {
    type Error = <BoxBody as MessageBody>::Error;

    fn size(&self) -> BodySize {
        self.body.size()
    }

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        let this = self.project();

        match this.body.poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                if this.body_buffer.len() < *this.max_size {
                    this.body_buffer.extend_from_slice(&chunk);
                }
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S, B> Service<ServiceRequest> for CacheMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let cache = self.cache.clone();
        let config = self.config.clone();
        let service = self.service.clone();

        Box::pin(async move {
            let body = if config.requires_body {
                req.take_payload()
                    .fold(BytesMut::new(), move |mut acc, chunk| async {
                        if let Ok(chunk) = chunk {
                            acc.extend_from_slice(&chunk);
                        }
                        acc
                    })
                    .await
            } else {
                BytesMut::new()
            };

            

            let ctx = RequestContext {
                method: req.method().as_str(),
                path: req.path(),
                query_string: req.query_string(),
                headers: req.headers(),
                body: &LazyBody::new(body.clone()),
            };

            let cache_key = (config.key_gen_fn)(&ctx);
            let should_cache = (config.cache_condition)(&ctx);

            if should_cache {
                match cache.get(&cache_key, config.ttl).await {
                    Ok(Some(entry)) => {
                        let mut res = Response::build(
                            StatusCode::from_u16(entry.status).unwrap_or(StatusCode::OK)
                        );

                        for (header, value) in entry.headers {
                            res.insert_header((header, value));
                        }

                        res.body(entry.body);

                        return Ok(
                            req.into_response(res)
                                .map_body(|_, body| EitherBody::right(body.boxed()))
                        );
                    },
                    Ok(None) => (),
                    Err(e) => tracing::error!("Failed to get value from cache: {:?}", e),
                };
            }

            if config.requires_body {
                req.set_payload(actix_web::dev::Payload::from(body.freeze()));
            }

            let fut = CacheResponseFuture::<S, B> {
                fut: service.call(req),
                should_cache,
                cache_key,
                cache: cache,
                ttl: config.ttl,
                max_cache_size: config.max_cache_size,
                _marker: PhantomData,
            };

            fut.await
        })
    }
}