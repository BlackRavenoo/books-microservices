use serde::Deserialize;
use secrecy::{ExposeSecret, SecretBox};
use sea_orm::ConnectOptions;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub search: SearchSettings,
    pub cache: CacheSettings,
    pub s3: S3Settings
}

#[derive(Deserialize, Debug)]
pub struct SearchSettings {
    pub url: String,
    pub books_index_name: String,
    pub authors_index_name: String
}

#[derive(Deserialize, Debug)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub protocol: String,
    pub username: String,
    pub password: SecretBox<String>,
    pub host: String,
    pub database_name: String
}

#[derive(Deserialize, Debug)]
pub struct CacheSettings {
    pub url: String
}

#[derive(Deserialize, Debug)]
pub struct S3Settings {
    pub access_key: SecretBox<String>,
    pub secret_key: SecretBox<String>,
    pub region: String,
    pub endpoint: String,
    pub name: String
}

impl DatabaseSettings {
    pub fn get_options(&self) -> ConnectOptions {
        let url = format!(
            "{}://{}:{}@{}/{}",
            self.protocol,
            self.username,
            self.password.expose_secret(),
            self.host,
            self.database_name
        ); 
        let mut options = ConnectOptions::new(url);

        options.connect_lazy(true);

        options
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let config_dir = base_path.join("configuration");

    let env: Environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENV");

    let settings = config::Config::builder()
        .add_source(config::File::from(config_dir.join("base")).required(true))
        .add_source(config::File::from(config_dir.join(env.as_str())).required(true))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment{
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(format!("{} is not a supported environment", other)),
        }
    }
}