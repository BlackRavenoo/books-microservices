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
    pub genres_include: Option<Vec<i16>>,
    pub genres_exclude: Option<Vec<i16>>,
    pub tags_include: Option<Vec<i16>>,
    pub tags_exclude: Option<Vec<i16>>,
    pub statuses: Option<Vec<i16>>,
}

#[derive(Deserialize, Serialize)]
pub struct SearchQuery {
    q: String
}

#[derive(Serialize, Deserialize)]
pub struct InputChapterSchema {
    pub number: i64
}

#[derive(Deserialize)]
pub struct BookRatingSchema {
    pub book_id: i32,
    pub avg_rating: f32
}

#[derive(Serialize)]
pub struct UserIdSchema {
    pub user_id: Option<i32>
}

#[derive(Deserialize)]
pub struct RateInputSchema {
    pub score: i16,
    pub item_id: i32,
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
    pub avg: f32,
    pub user: Option<i16>
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
    pub first_chapter_key: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<Rating>
}

#[derive(Serialize, Deserialize)]
pub struct PaginationSchema<T> {
    pub max_page: u64,
    pub total_items: u64,
    pub items: Vec<T>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BookSchema {
    pub id: i32,
    pub title: String,
    pub thumbnail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_rating: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<BookStatus>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConstantsSchema {
    pub tags: Vec<Tag>,
    pub genres: Vec<Genre>,
    pub status: Vec<BookStatus>,
}

#[derive(Deserialize, Serialize)]
pub struct ChapterFullSchema {
    pub id: i64,
    pub index: i16,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
    pub book_id: i32,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct BulkGetSchema {
    pub ids: Vec<i32>
}

#[derive(Serialize)]
pub struct RateOutputSchema {
    pub score: i16,
    pub item_id: i32,
    pub user_id: i32
}