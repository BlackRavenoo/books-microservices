use elasticsearch::{
    http::{
        transport::{SingleNodeConnectionPool, TransportBuilder},
        Url
    },
    indices::{IndicesCreateParts, IndicesExistsParts},
    Elasticsearch,
    Error,
    SearchParts
};
use serde_json::{json, Value};

pub struct ElasticsearchClient {
    elasticseach: Elasticsearch,
    index: String
}

pub type IdType = i32;

#[derive(Debug, thiserror::Error)]
pub enum ElasticError {
    #[error("Elasticsearch request failed: {0}")]
    RequestError(#[from] Error),

    #[error("Index creation failed: {0}")]
    IndexCreationError(String),
}

impl ElasticsearchClient {
    pub fn new(url:  &str, index: &str) -> Self {
        let url = Url::parse(url).expect("Wrong url");

        let conn_pool = SingleNodeConnectionPool::new(url);
        let builder = TransportBuilder::new(conn_pool);
        let transport = builder.build().unwrap();

        Self {
            elasticseach: Elasticsearch::new(transport),
            index: index.to_owned()
        }
    }

    pub async fn create_index(&self) -> Result<(), ElasticError> {
        let is_exists = self.elasticseach
            .indices()
            .exists(IndicesExistsParts::Index(&[&self.index]))
            .send()
            .await
            .map_err(ElasticError::RequestError)?
            .status_code()
            .is_success();

        if is_exists {
            tracing::debug!("Index {} already exists", self.index);
            return Ok(())
        }

        tracing::info!("Creating index {}", self.index);

        let response = self.elasticseach
            .indices()
            .create(IndicesCreateParts::Index(&self.index))
            .body(json!({
                "settings": {
                    "analysis": {
                        "analyzer": {
                            "russian_analyzer": {
                                "type": "custom",
                                "tokenizer": "standard",
                                "filter": [
                                    "lowercase",
                                    "russian_stop",
                                    "russian_stemmer"
                                ]
                            }
                        },
                        "filter": {
                            "russian_stop": {
                                "type": "stop",
                                "stopwords": "_russian_"
                            },
                            "russian_stemmer": {
                                "type": "stemmer",
                                "language": "russian"
                            }
                        }
                    }
                },
                "mappings": {
                    "properties": {
                        "id": { "type": "integer" },
                        "title": {
                            "type": "text",
                            "analyzer": "russian_analyzer",
                            "fields": {
                                "keyword": {
                                    "type": "keyword",
                                    "ignore_above": 256
                                }
                            }
                        }
                    }
                }
            }))
            .send()
            .await
            .map_err(ElasticError::RequestError)?;

        let status = response.status_code();

        if !status.is_success() {
            let json = response.json::<Value>().await
                .map_err(ElasticError::RequestError)?;
            return Err(ElasticError::IndexCreationError(format!("{:?}", json)));
        }
        
        Ok(())
    }

    // TODO: Option fields
    pub async fn search(&self, title: &str) -> Result<IdType, Error> {
        let response = self.elasticseach
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({
                "query": {
                    "match": {
                        "title": title
                    }
                },
                "_source": ["id", "title", "cover"]
            }))
            .send()
            .await?;

        let json = response.json::<Value>().await?;

        println!("{:?}", json);
        
        todo!()
    }
}