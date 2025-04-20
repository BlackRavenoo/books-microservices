use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub enum OrderBy {
    Rating,
    ChaptersCount,
    CreatedAt,
    NameDesc,
    NameAsc
}

#[derive(Deserialize)]
pub struct GetListSchema {
    pub page: Option<u64>,
    pub page_size: Option<u64>
}

#[derive(Deserialize)]
pub struct GetBookSchema {
    pub id: i32
}

// Output schema

#[derive(Serialize, Deserialize, Clone, Decode, Encode)]
pub struct Tag {
    pub id: i16,
    pub name: String
}

#[derive(Serialize, Deserialize, Clone, Decode, Encode)]
pub struct Genre {
    pub id: i16,
    pub name: String
}

#[derive(Serialize, Deserialize, Clone, Decode, Encode)]
pub struct BookStatusWithName {
    id: i16,
    name: String,
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
    pub status: BookStatusWithName,
    pub cover: String,
    pub created_at: String,
    pub tags: Vec<Tag>,
    pub genres: Vec<Genre>,
    pub authors: Vec<Author>,
    pub chapters_count: i16
}

#[derive(Serialize, Deserialize, Clone, Decode, Encode)]
pub struct BookSchema {
    pub id: i32,
    pub title: String,
    pub thumbnail: String,
}

#[derive(Serialize)]
pub struct AuthorSchema {
    pub id: i32,
    pub name: String,
    pub cover: String,
    pub books: Vec<BookSchema>
}

#[derive(Serialize, Deserialize, Clone, Decode, Encode)]
pub struct ConstantsSchema {
    pub tags: Vec<Tag>,
    pub genres: Vec<Genre>,
    pub status: Vec<BookStatusWithName>,
}