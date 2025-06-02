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
pub struct GetListSchema {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
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

#[derive(Serialize)]
pub struct PaginationSchema<T> {
    pub max_page: u64,
    pub total_items: u64,
    pub items: Vec<T>
}