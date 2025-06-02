use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct BulkGetSchema {
    pub ids: Vec<i32>
}

#[derive(Deserialize)]
pub struct GetSchema {
    pub user_id: Option<i32>
}

#[derive(Deserialize)]
pub struct RateSchema {
    pub score: i16,
    pub item_id: i32,
    pub user_id: i32
}

#[derive(Serialize)]
pub struct BookRatingSchema {
    pub book_id: i32,
    pub avg_rating: f32
}

#[derive(Serialize)]
pub struct RatingSchema {
    pub avg: f32,
    pub user: Option<i16>
}