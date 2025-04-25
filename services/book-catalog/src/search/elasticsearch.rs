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

use crate::schema::BookSchema;

pub struct ElasticsearchClient {
    elasticseach: Elasticsearch,
    index: String
}

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
                            },
                            "ngram_analyzer": {
                                "type": "custom",
                                "tokenizer": "standard",
                                "filter": [
                                    "lowercase",
                                    "russian_stop",
                                    "russian_ngram"
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
                            },
                            "russian_ngram": {
                                "type": "ngram",
                                "min_gram": 3,
                                "max_gram": 4
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
                                },
                                "ngram": {
                                    "type": "text",
                                    "analyzer": "ngram_analyzer"
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
    pub async fn search(&self, query: &str) -> Result<Vec<BookSchema>, Error> {
        let response = self.elasticseach
            .search(SearchParts::Index(&[&self.index]))
            .body(json!({
                "query": {
                    "multi_match": {
                        "query": query,
                        "fields": ["title", "title.ngram"]
                    }
                },
                "_source": ["id", "title", "cover"]
            }))
            .send()
            .await?;

        let json = response.json::<Value>().await?;
        
        match json.get("hits") {
            Some(hits) => match hits.get("hits") {
                Some(hits_array) => {
                    let hits_array = match hits_array.as_array() {
                        Some(arr) => arr,
                        None => {
                            return Ok(Vec::new())
                        },
                    };
                    
                    let mut results = Vec::with_capacity(hits_array.len());
                    
                    for hit in hits_array {
                        if let Some(source) = hit.get("_source") {
                            if let Ok(book) = serde_json::from_value(source.clone()) {
                                results.push(book);
                            }
                        }
                    }
                    
                    Ok(results)
                },
                None => Ok(Vec::new()),
            },
            None => Ok(Vec::new()),
        }
    }
}