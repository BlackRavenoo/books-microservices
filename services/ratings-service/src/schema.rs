use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct BulkGetSchema {
    pub ids: Vec<i32>
}

#[derive(Serialize)]
pub struct BooksRatingSchema {
    pub book_id: i32,
    pub avg_rating: String
}