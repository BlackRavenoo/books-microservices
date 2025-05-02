use sqlx::PgPool;
use anyhow::anyhow;

use crate::{auth::password::{self, verify_password}, schema::User};

pub struct UserService {
    db_pool: PgPool,
}

impl UserService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
    
    pub async fn authenticate(&self, email: &str, password_input: String) -> anyhow::Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, name, email, password_hash FROM users WHERE email = $1"#,
            email
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| anyhow!("Пользователь не найден"))?;
        
        verify_password(password_input, &user.password_hash)?;
        
        Ok(user)
    }
    
    pub async fn register(&self, name: &str, email: &str, password: &str) -> anyhow::Result<i32> {
        let existing_user = sqlx::query!(
            r#"SELECT id FROM users WHERE email = $1"#,
            email
        )
        .fetch_optional(&self.db_pool)
        .await?;
        
        if existing_user.is_some() {
            return Err(anyhow!("A user with this email already exists"));
        }
        
        let password_hash = password::hash_password(password.to_string())?;
        
        let user_id = sqlx::query!(
            r#"
            INSERT INTO users (name, email, password_hash) 
            VALUES ($1, $2, $3) 
            RETURNING id
            "#,
            name,
            email,
            password_hash
        )
        .fetch_one(&self.db_pool)
        .await?
        .id;
        
        Ok(user_id)
    }

    pub async fn get_user_roles(&self, user_id: i32) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar!(
            "SELECT r.name FROM roles r
            JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1",
            user_id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn get_username(&self, user_id: i32) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!(
            "SELECT name FROM users
            WHERE id = $1",
            user_id
        )
        .fetch_one(&self.db_pool)
        .await
    }
}