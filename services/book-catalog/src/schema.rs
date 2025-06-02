use actix_multipart::form::{json::Json, tempfile::TempFile, MultipartForm};
use bincode::{Decode, Encode};
use sea_orm::{prelude::DateTimeWithTimeZone, DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize, Serializer};

use crate::entity::{book::{self, BookStatus}, tag, genre, chapter};

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub enum Target {
    #[serde(rename = "author")]
    Author,
    #[serde(rename = "series")]
    Series,
}

#[derive(Deserialize)]
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
    pub statuses: Option<Vec<BookStatus>>,
}

#[derive(Deserialize)]
pub struct GetBookSchema {
    pub id: i32
}

#[derive(Deserialize)]
pub struct SearchSchema {
    pub q: String
}

#[derive(Deserialize)]
pub struct CreateBookSchema {
    pub title: String,
    pub description: String,
    pub status: BookStatus,
    pub series_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<i16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<i16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<i32>,
}

#[derive(MultipartForm)]
pub struct CreateBookForm {
    #[multipart(limit = "5MB")]
    pub cover: TempFile,
    pub fields: Json<CreateBookSchema>
}

#[derive(Deserialize)]
pub struct UpdateBookSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<BookStatus>,
    pub series_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags_to_delete: Vec<i16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genres_to_delete: Vec<i16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors_to_delete: Vec<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags_to_add: Vec<i16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genres_to_add: Vec<i16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors_to_add: Vec<i32>
}

#[derive(MultipartForm)]
pub struct UpdateBookForm {
    #[multipart(limit = "5MB")]
    pub cover: Option<TempFile>,
    pub fields: Json<UpdateBookSchema>
}

#[derive(Deserialize)]
pub struct CreateAuthorSchema {
    pub name: String
}

#[derive(MultipartForm)]
pub struct CreateAuthorForm {
    #[multipart(limit = "5MB")]
    pub cover: TempFile,
    pub fields: Json<CreateAuthorSchema>
}

#[derive(Deserialize)]
pub struct UpdateAuthorSchema {
    pub name: Option<String>
}

#[derive(MultipartForm)]
pub struct UpdateAuthorForm {
    #[multipart(limit = "5MB")]
    pub cover: Option<TempFile>,
    pub fields: Json<UpdateAuthorSchema>
}

#[derive(Deserialize)]
pub struct CreateChapterFields {
    pub name: String,
    pub content: serde_json::Value,
    pub index: i16,
}

#[derive(Deserialize)]
pub struct UpdateChapterFields {
    pub name: Option<String>,
    pub content: Option<serde_json::Value>,
    pub index: Option<i16>
}

#[derive(Deserialize)]
pub struct InputChapterSchema {
    pub number: i64
}

#[derive(Deserialize)]
pub struct BulkGetSchema {
    pub ids: Vec<i32>
}

// Output schema

#[derive(Serialize, Deserialize, Clone, Decode, Encode, FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "tag::Entity")]
pub struct Tag {
    pub id: i16,
    pub name: String
}

#[derive(Serialize, Deserialize, Clone, Decode, Encode, FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "genre::Entity")]
pub struct Genre {
    pub id: i16,
    pub name: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BookStatusWithName {
    pub id: i16,
    pub name: &'static str,
}

#[derive(Serialize, Deserialize, Clone, Decode, Encode)]
pub struct Author {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Decode, Encode)]
pub struct BookFullSchema {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub status: BookStatus,
    pub cover: String,
    pub tags: Vec<Tag>,
    pub genres: Vec<Genre>,
    pub authors: Vec<Author>,
    pub chapters_count: i16
}

#[derive(Serialize)]
pub struct PaginationSchema<T> {
    pub max_page: u64,
    pub total_items: u64,
    pub items: Vec<T>
}

#[derive(Serialize, Deserialize, Clone, Decode, Encode, FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "book::Entity")]
pub struct BookSchema {
    pub id: i32,
    pub title: String,
    #[sea_orm(from_col = "cover")]
    #[serde(rename(deserialize = "cover"))]
    pub thumbnail: String,
}

#[derive(Deserialize, Serialize)]
pub struct AuthorSchema {
    pub id: i32,
    pub name: String,
    pub cover: String,
}

#[derive(Serialize, Deserialize, Clone, Decode, Encode)]
pub struct ConstantsSchema {
    pub tags: Vec<Tag>,
    pub genres: Vec<Genre>,
    pub status: Vec<BookStatus>,
}

impl Serialize for BookStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status_struct = BookStatusWithName {
            name: self.as_str(),
            id: self.clone() as i16,
        };
        
        status_struct.serialize(serializer)
    }
}

#[derive(Debug, Serialize, FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "chapter::Entity")]
pub struct ChapterFullSchema {
    pub id: i64,
    pub index: i16,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sea_orm(skip)]
    pub content: Option<serde_json::Value>,
    pub book_id: i32,
    pub created_at: DateTimeWithTimeZone,
}