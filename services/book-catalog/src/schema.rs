use bincode::{Decode, Encode};
use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize, Serializer};

use crate::entity::book::{self, BookStatus};

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

#[derive(Deserialize)]
pub struct SearchBookSchema {
    pub q: String
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

#[derive(Serialize, Deserialize, Clone)]
pub struct BookStatusWithName {
    id: i16,
    name: &'static str,
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

#[derive(Serialize, Deserialize, Clone, Decode, Encode, FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "book::Entity")]
pub struct BookSchema {
    pub id: i32,
    pub title: String,
    #[sea_orm(from_col = "cover")]
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