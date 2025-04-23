use s3::{creds::Credentials, Bucket};
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::config::S3Settings;

pub struct S3StorageBackend {
    bucket: Box<Bucket>,
}

// TODO: image_type(jpg, png...)

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
        .unwrap()
        .with_path_style();

        Self {
            bucket
        }
    }

    fn generate_key(&self, id: u32, image_id: Uuid) -> String {
        format!("{}/{}.jpg", id, image_id)
    }

    pub async fn save(
        &self,
        id: u32,
        image_id: Uuid,
        data: Vec<u8>
    ) -> anyhow::Result<()> {
        let key = self.generate_key(id, image_id);
        
        let content_type = "image/jpeg".to_string();
        
        let data = self.bucket
            .put_object_with_content_type(&key, &data, &content_type)
            .await?;

        let code = data.status_code();
        
        if code != 200 {
            let msg = format!("Failed to upload object, status code: {}", code);
            tracing::error!(msg);
            return Err(anyhow::anyhow!(msg));
        }

        Ok(())
    }

    pub async fn delete(
        &self,
        id: u32,
        image_id: Uuid,
    ) -> anyhow::Result<()> {
        let key = self.generate_key(id, image_id);

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

    pub fn get_url(
        &self,
        id: u32,
        image_id: Uuid
    ) -> String {
        format!(
            "{}/{}",
            self.bucket.url(),
            self.generate_key(id, image_id)
        )
    }  
}