use std::time::Duration;

use reqwest::{redirect::Policy, Client};

use crate::{config::ServicesSettings, error::ApiError, schema::{BookSchema, BooksListQuery}};

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

    pub async fn get_books_list(&self, query: &BooksListQuery) -> Result<Vec<BookSchema>, ApiError> {
        let url = format!("{}/api/v1/books", self.config.book_catalog.url);
        
        let response = self.client.get(&url)
            .query(query)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to call book-catalog service: {:?}", e);
                ApiError::ServiceError(format!("Failed to connect to book catalog service: {}", e))
            })?;
            
        if !response.status().is_success() {
            return Err(ApiError::ServiceError(format!(
                "Book catalog service returned error status: {}", 
                response.status()
            )));
        }
        
        response.json::<Vec<BookSchema>>()
            .await
            .map_err(|e| {
                tracing::error!("Failed to deserialize book catalog response: {:?}", e);
                ApiError::ServiceError(format!("Invalid response from book catalog service: {}", e))
            })
    }
}