CREATE TABLE IF NOT EXISTS analytics.ratings_raw (
    id UInt64,
    user_id UInt32,
    book_id UInt32,
    rating UInt8,
    updated_at DateTime,
    _kafka_offset UInt64,
    _kafka_partition UInt8,
    _kafka_timestamp DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (book_id, user_id, updated_at)
PARTITION BY toYYYYMM(updated_at);

CREATE TABLE IF NOT EXISTS analytics.book_stats_raw (
    book_id UInt32,
    total_ratings UInt32,
    sum_ratings UInt64,
    avg_rating Decimal(4,2) MATERIALIZED 
        CASE WHEN total_ratings > 0 
             THEN sum_ratings / total_ratings 
             ELSE 0 END,
    _kafka_offset UInt64,
    _kafka_partition UInt8,
    _kafka_timestamp DateTime DEFAULT now()
) ENGINE = ReplacingMergeTree(_kafka_timestamp)
ORDER BY book_id;

CREATE TABLE IF NOT EXISTS analytics.ratings_daily_stats (
    date Date,
    book_id UInt32,
    total_ratings UInt32,
    sum_ratings UInt64,
    avg_rating Float64,
    rating_1 UInt32,
    rating_2 UInt32,
    rating_3 UInt32,
    rating_4 UInt32,
    rating_5 UInt32,
    rating_6 UInt32,
    rating_7 UInt32,
    rating_8 UInt32,
    rating_9 UInt32,
    rating_10 UInt32
) ENGINE = SummingMergeTree()
ORDER BY (date, book_id)
PARTITION BY toYYYYMM(date);

CREATE MATERIALIZED VIEW IF NOT EXISTS analytics.ratings_daily_mv TO analytics.ratings_daily_stats AS
SELECT
    toDate(updated_at) as date,
    book_id,
    1 as total_ratings,
    rating as sum_ratings,
    rating as avg_rating,
    rating = 1 ? 1 : 0 as rating_1,
    rating = 2 ? 1 : 0 as rating_2,
    rating = 3 ? 1 : 0 as rating_3,
    rating = 4 ? 1 : 0 as rating_4,
    rating = 5 ? 1 : 0 as rating_5,
    rating = 6 ? 1 : 0 as rating_6,
    rating = 7 ? 1 : 0 as rating_7,
    rating = 8 ? 1 : 0 as rating_8,
    rating = 9 ? 1 : 0 as rating_9,
    rating = 10 ? 1 : 0 as rating_10
FROM analytics.ratings_raw;

CREATE VIEW IF NOT EXISTS analytics.top_books AS
SELECT
    book_id,
    sum(total_ratings) as total_ratings_sum,
    sum(sum_ratings) / sum(total_ratings) as avg_rating,
    sum(rating_1) as rating_1_count,
    sum(rating_2) as rating_2_count,
    sum(rating_3) as rating_3_count,
    sum(rating_4) as rating_4_count,
    sum(rating_5) as rating_5_count,
    sum(rating_6) as rating_6_count,
    sum(rating_7) as rating_7_count,
    sum(rating_8) as rating_8_count,
    sum(rating_9) as rating_9_count,
    sum(rating_10) as rating_10_count
FROM analytics.ratings_daily_stats
GROUP BY book_id
HAVING total_ratings_sum >= 1
ORDER BY avg_rating DESC;

CREATE VIEW IF NOT EXISTS analytics.daily_trends AS
SELECT
    date,
    sum(total_ratings) as total_ratings,
    avg(avg_rating) as avg_rating,
    count(DISTINCT book_id) as unique_books
FROM analytics.ratings_daily_stats
GROUP BY date
ORDER BY date DESC;