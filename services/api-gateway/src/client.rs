use std::{sync::Arc, time::Duration};

use reqwest::{redirect::Policy, Client};

use crate::config::ServicesSettings;

pub struct ServiceClient {
    client: Client,
    config: Arc<ServicesSettings>,
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
            config: Arc::new(settings),
        }
    }


}