use anyhow::{Result, Context};
use sqlx::PgPool;

use crate::schema::Client;

pub struct ClientStore {
    pool: PgPool,
}

impl ClientStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    pub async fn get_client(&self, client_id: &str) -> Result<Option<Client>> {
        let client_data = sqlx::query!(
            r#"
            SELECT id, name
            FROM clients
            WHERE id = $1
            "#,
            client_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch client")?;
        
        if let Some(client) = client_data {
            let redirect_uris = sqlx::query!(
                r#"
                SELECT redirect_uri
                FROM client_redirect_uris
                WHERE client_id = $1
                "#,
                client_id
            )
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch client redirect URIs")?
            .into_iter()
            .map(|row| row.redirect_uri)
            .collect();
            
            return Ok(Some(Client {
                id: client.id,
                name: client.name,
                redirect_uris,
            }));
        }
        
        Ok(None)
    }
    
    pub async fn create_client(&self, client: Client) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        
        sqlx::query!(
            r#"
            INSERT INTO clients (id, name)
            VALUES ($1, $2)
            "#,
            client.id,
            client.name
        )
        .execute(&mut *tx)
        .await
        .context("Failed to insert client")?;
        
        for uri in client.redirect_uris {
            sqlx::query!(
                r#"
                INSERT INTO client_redirect_uris (client_id, redirect_uri)
                VALUES ($1, $2)
                "#,
                client.id,
                uri
            )
            .execute(&mut *tx)
            .await
            .context("Failed to insert client redirect URI")?;
        }
        
        tx.commit().await.context("Failed to commit transaction")?;
        Ok(())
    }

    pub async fn client_exists(&self, client_id: &str) -> Result<bool> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM clients WHERE id = $1)",
            client_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to check if client exists")?;
        
        Ok(exists.unwrap_or(false))
    }
}