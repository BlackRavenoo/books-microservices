use serde::{Deserialize, Serialize};

// Input

#[derive(Serialize, Deserialize)]
pub enum OrderBy {
    #[serde(rename = "chap_count")]
    ChaptersCount,
    #[serde(rename = "created_at")]
    CreatedAt,
    #[serde(rename = "name_desc")]
    NameDesc,
    #[serde(rename = "name_asc")]
    NameAsc
}

#[derive(Serialize, Deserialize)]
pub enum Target {
    #[serde(rename = "author")]
    Author,
    #[serde(rename = "series")]
    Series,
}

#[derive(Serialize, Deserialize)]
pub struct GetListSchema {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub order_by: Option<OrderBy>,
    pub target: Option<Target>,
    pub target_id: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct SearchQuery {
    q: String
}

// Output

#[derive(Serialize, Deserialize, Clone)]
pub struct BookStatus {
    id: i16,
    name: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i16,
    pub name: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Genre {
    pub id: i16,
    pub name: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Author {
    pub id: i32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Rating {
    pub average: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BookFullSchema {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub status: BookStatus,
    pub cover: String,
    pub tags: Vec<Tag>,
    pub genres: Vec<Genre>,
    pub authors: Vec<Author>,
    pub chapters_count: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<Rating>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BookSchema {
    pub id: i32,
    pub title: String,
    pub thumbnail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<Rating>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConstantsSchema {
    pub tags: Vec<Tag>,
    pub genres: Vec<Genre>,
    pub status: Vec<BookStatus>,
}