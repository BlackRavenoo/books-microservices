-- Add migration script here
CREATE TABLE ratings (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id INT NOT NULL,
    book_id INT NOT NULL,
    rating INT NOT NULL CHECK (rating >= 1 AND rating <= 10),
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, book_id)
);

CREATE INDEX idx_ratings_user_id ON ratings(user_id);
CREATE INDEX idx_ratings_book_id ON ratings(book_id);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_ratings_updated_at BEFORE UPDATE ON ratings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE book_rating_stats (
    book_id INT PRIMARY KEY,
    total_ratings INT NOT NULL DEFAULT 0,
    sum_ratings BIGINT NOT NULL DEFAULT 0,
    avg_rating DECIMAL(3,2) GENERATED ALWAYS AS (
        CASE WHEN total_ratings > 0 THEN sum_ratings::DECIMAL / total_ratings ELSE 0 END
    ) STORED NOT NULL
);

CREATE INDEX idx_book_rating_stats_avg ON book_rating_stats(avg_rating DESC);

CREATE OR REPLACE FUNCTION update_book_rating_stats()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO book_rating_stats (book_id, total_ratings, sum_ratings)
        VALUES (NEW.book_id, 1, NEW.rating)
        ON CONFLICT (book_id) DO UPDATE SET
            total_ratings = book_rating_stats.total_ratings + 1,
            sum_ratings = book_rating_stats.sum_ratings + NEW.rating;
    ELSIF TG_OP = 'UPDATE' THEN
        UPDATE book_rating_stats SET
            sum_ratings = sum_ratings - OLD.rating + NEW.rating
        WHERE book_id = NEW.book_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE book_rating_stats SET
            total_ratings = total_ratings - 1,
            sum_ratings = sum_ratings - OLD.rating
        WHERE book_id = OLD.book_id;
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$$ language 'plpgsql';

CREATE TRIGGER update_book_stats_on_rating_change
    AFTER INSERT OR UPDATE OR DELETE ON ratings
    FOR EACH ROW EXECUTE FUNCTION update_book_rating_stats();

