use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

use crate::entity::book::BookStatus;

#[derive(Deserialize)]
pub struct GetListSchema {
    pub page: Option<u64>,
    pub page_size: Option<u64>
}

#[derive(Deserialize)]
pub struct GetBookSchema {
    pub id: i32
}






#[derive(Serialize)]
pub struct Tag {
    pub id: i16,
    pub name: String
}

#[derive(Serialize)]
pub struct Genre {
    pub id: i16,
    pub name: String
}

#[derive(Serialize)]
pub struct BookStatusWithName {
    id: BookStatus,
    name: &'static str
}

#[derive(Serialize)]
pub struct Author {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize)]
pub struct BookFullSchema {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub status: BookStatusWithName,
    pub cover: String,
    pub created_at: DateTimeWithTimeZone,
    pub tags: Vec<Tag>,
    pub genres: Vec<Genre>,
    pub authors: Vec<Author>,
    pub chapters_count: i16
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct ConstantsSchema {
    pub tags: Vec<Tag>,
    pub genres: Vec<Genre>,
    pub status: Vec<BookStatusWithName>,
}