use std::{str::FromStr, time::Duration};

use actix_web::{dev::PeerAddr, error, web, Error, HttpRequest, HttpResponse};
use futures_util::StreamExt as _;
use reqwest::{redirect::Policy, Client, Url};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{config::ServicesSettings, error::ApiError, schema::{Author, BookFullSchema, BookSchema, ChapterFullSchema, InputChapterSchema, ConstantsSchema, GetListSchema, PaginationSchema, SearchQuery}};

pub struct ServiceClient {
    client: Client,
    config: ServicesSettings,
    // cache maybe
}

impl ServiceClient {
    pub fn new(settings: ServicesSettings) -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .redirect(Policy::none())
            .user_agent("Book_API_Gateway/0.1")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config: settings,
        }
    }

    pub async fn get_books_list(&self, query: &GetListSchema) -> Result<PaginationSchema<BookSchema>, ApiError> {
        let query_string = match serde_qs::to_string(query) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to get query string from GetListSchema: {:?}", e);
                return Err(ApiError::ValidationError)
            },
        };
        let url = format!("{}/api/v1/books?{}", self.config.book_catalog.url, query_string);
        self.make_request(&url, &self.config.book_catalog.name, reqwest::Method::GET, None::<&()>).await
    }
    
    pub async fn get_book(&self, id: u64) -> Result<BookFullSchema, ApiError> {
        let url = format!("{}/api/v1/books/{}", self.config.book_catalog.url, id);
        self.make_request(&url, &self.config.book_catalog.name, reqwest::Method::GET, None::<&()>).await
    }

    pub async fn update_entity(
        &self,
        req: HttpRequest,
        payload: web::Payload,
        peer_addr: Option<PeerAddr>
    ) -> Result<HttpResponse, Error> {
        let url = format!("{}/api/v1{}", self.config.book_catalog.url, req.uri().path());

        self.forward_request(
            req,
            payload,
            actix_web::http::Method::PUT,
            peer_addr,
            &url
        ).await
    }

    pub async fn create_entity(
        &self,
        req: HttpRequest,
        payload: web::Payload,
        peer_addr: Option<PeerAddr>
    ) -> Result<HttpResponse, Error> {
        let url = format!("{}/api/v1{}", self.config.book_catalog.url, req.uri().path());

        self.forward_request(
            req,
            payload,
            actix_web::http::Method::POST,
            peer_addr,
            &url
        ).await
    }

    pub async fn delete_entity(
        &self,
        req: HttpRequest
    ) -> Result<(), ApiError> {
        let url = format!("{}/api/v1{}", self.config.book_catalog.url, req.uri().path());
        self.make_request(&url, &self.config.book_catalog.name, reqwest::Method::DELETE, None::<&()>).await
    }

    pub async fn search<T>(&self, q: SearchQuery, entity: &str) -> Result<Vec<T>, ApiError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let url = format!("{}/api/v1/search/{}", self.config.book_catalog.url, entity);
        self.make_request(&url, &self.config.book_catalog.name, reqwest::Method::GET, Some(&q)).await
    }

    async fn forward_request(
        &self,
        req: HttpRequest,
        mut payload: web::Payload,
        method: actix_web::http::Method,
        peer_addr: Option<PeerAddr>,
        url: &str
    ) -> Result<HttpResponse, Error> {
        let mut new_url = match Url::from_str(&url) {
            Ok(url) => url,
            Err(e) => {
                tracing::error!("Failed to create url from str: {:?}", e);
                return Ok(HttpResponse::InternalServerError().finish())
            },
        };

        new_url.set_query(req.uri().query());

        let (tx, rx) = mpsc::unbounded_channel();

        actix_web::rt::spawn(async move {
            while let Some(chunk) = payload.next().await {
                tx.send(chunk).unwrap();
            }
        });
    
        let mut forwarded_req = self.client
            .request(
                reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap(),
                new_url,
            )
            .body(reqwest::Body::wrap_stream(UnboundedReceiverStream::new(rx)));

        for (name, value) in req.headers().iter() {
            forwarded_req = forwarded_req.header(name.as_str(), value.as_bytes());
        }

        let forwarded_req = match peer_addr {
            Some(PeerAddr(addr)) => forwarded_req.header("x-forwarded-for", addr.ip().to_string()),
            None => forwarded_req,
        };
    
        let res = forwarded_req
            .send()
            .await
            .map_err(error::ErrorInternalServerError)?;

            let mut client_resp =
            HttpResponse::build(actix_web::http::StatusCode::from_u16(res.status().as_u16()).unwrap());

        for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
            client_resp.insert_header((
                actix_web::http::header::HeaderName::from_bytes(header_name.as_ref()).unwrap(),
                actix_web::http::header::HeaderValue::from_bytes(header_value.as_ref()).unwrap(),
            ));
        }

        Ok(client_resp.streaming(res.bytes_stream()))
    }

    pub async fn get_constants(&self) -> Result<ConstantsSchema, ApiError> {
        let url = format!("{}/api/v1/constants", self.config.book_catalog.url);
        self.make_request(&url, &self.config.book_catalog.name, reqwest::Method::GET, None::<&()>).await
    }

    pub async fn get_author(&self, id: u64) -> Result<Author, ApiError> {
        let url = format!("{}/api/v1/authors/{}", self.config.book_catalog.url, id);
        self.make_request(&url, &self.config.book_catalog.name, reqwest::Method::GET, None::<&()>).await
    }

    pub async fn get_chapter(&self, book_id: u64, chapter_id: InputChapterSchema) -> Result<ChapterFullSchema, ApiError> {
        let url = format!("{}/api/v1/books/{}/chapter", self.config.book_catalog.url, book_id);
        self.make_request(&url, &self.config.book_catalog.name, reqwest::Method::GET, Some(&chapter_id)).await
    }

    pub async fn get_chapters_list(&self, book_id: u64) -> Result<Vec<ChapterFullSchema>, ApiError> {
        let url = format!("{}/api/v1/books/{}/chapters", self.config.book_catalog.url, book_id);
        self.make_request(&url, &self.config.book_catalog.name, reqwest::Method::GET, None::<&()>).await
    }

    #[inline]
    async fn make_request<T, Q>(&self, url: &str, service_name: &str, method: reqwest::Method, query: Option<&Q>) -> Result<T, ApiError>
    where
        T: for<'de> serde::Deserialize<'de>,
        Q: serde::Serialize + ?Sized,
    {
        let method_str = method.as_str().to_owned();
        let mut request = self.client.request(method, url);
        
        if let Some(q) = query {
            request = request.query(q);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to call {} {}: {:?}", method_str, url, e);
                ApiError::ServiceError("Failed to make request".to_owned())
            })?;
            
        if response.status().is_success() {
            response.json::<T>()
                .await
                .map_err(|e| {
                    tracing::error!("Failed to deserialize response from {}: {:?}", service_name, e);
                    ApiError::ServiceError(format!("Invalid response from {}", service_name))
                })
        } else {
            Err(ApiError::ServiceError(format!(
                "{} returned error status: {}. Message: {}",
                service_name, 
                response.status(),
                response.text().await.unwrap_or_default()
            )))
        }
    }
}