use s3::{creds::Credentials, Bucket};
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::config::S3Settings;

pub struct S3StorageBackend {
    bucket: Box<Bucket>,
}

impl S3StorageBackend {
    pub fn new(config: S3Settings) -> Self {
        let bucket = Bucket::new(
            &config.name,
            s3::Region::Custom { region: config.region, endpoint: config.endpoint },
            Credentials::new(
                Some(config.access_key.expose_secret()),
                Some(config.secret_key.expose_secret()),
                None,
                None,
                None
            ).unwrap()
        )
        .unwrap();

        Self {
            bucket
        }
    }

    fn generate_key(&self, storage_id: u32, object_id: Uuid, extension: &str) -> String {
        format!("{}/{}.{}", storage_id, object_id, extension)
    }

    pub async fn save(
        &self,
        storage_id: u32,
        object_id: Uuid,
        data: Vec<u8>,
        content_type: &str,
        extension: &str
    ) -> anyhow::Result<String> {
        let key = self.generate_key(storage_id, object_id, extension);
        
        let response = self.bucket
            .put_object_with_content_type(&key, &data, content_type)
            .await?;

        let code = response.status_code();
        
        if code != 200 {
            let msg = format!("Failed to upload object, status code: {}", code);
            return Err(anyhow::anyhow!(msg));
        }

        Ok(format!("{}/{}", self.bucket.url(), key))
    }

    pub async fn get(&self, url: &str) -> anyhow::Result<Vec<u8>> {
        let key = url.split(&self.bucket.url())
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("Invalid URL format"))?
            .trim_start_matches('/');

        let response = self.bucket
            .get_object(key)
            .await?;

        let code = response.status_code();
        
        if code != 200 {
            let msg = format!("Failed to get object, status code: {}", code);
            return Err(anyhow::anyhow!(msg));
        }

        Ok(response.bytes().to_vec())
    }

    pub async fn delete(
        &self,
        id: u32,
        image_id: Uuid,
        extension: &str,
    ) -> anyhow::Result<()> {
        let key = self.generate_key(id, image_id, extension);

        let data = self.bucket
            .delete_object(&key)
            .await?;

        let code = data.status_code();
        
        if code != 204 {
            let msg = format!("Failed to delete object, status code: {}", code);
            tracing::error!(msg);
            return Err(anyhow::anyhow!(msg));
        }

        Ok(())
    }

    pub fn extract_uuid_from_url(&self, url: &str) -> Option<Uuid> {
        if let Some(last_part) = url.split('/').last() {
            if let Some(uuid_str) = last_part.split('.').next() {
                if let Ok(uuid) = Uuid::parse_str(uuid_str) {
                    return Some(uuid);
                }
            }
        }
        None
    }

    pub fn get_object_url(
        &self,
        storage_id: u32,
        object_id: Uuid,
        extension: &str
    ) -> String {
        format!(
            "{}/{}",
            self.bucket.url(),
            self.generate_key(storage_id, object_id, extension)
        )
    }

    pub fn get_image_url(
        &self,
        storage_id: u32,
        image_id: Uuid
    ) -> String {
        self.get_object_url(storage_id, image_id, "jpg")
    }
}