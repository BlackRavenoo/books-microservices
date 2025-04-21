use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Responder};
use sea_orm::{prelude::Expr, DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect, RelationTrait};

use crate::{
    entity::{
        book::{self, Entity as Book}, book_author, book_genre, book_tag
    },
    schema::{BookFullSchema, BookSchema, GetBookSchema, GetListSchema, SearchBookSchema}, search::ElasticsearchClient
};

const DEFAULT_PAGE_SIZE: u64 = 50;

// TODO: add custom order_by and custom fields
pub async fn get_books(db: web::Data<DatabaseConnection>, query: web::Query<GetListSchema>) -> impl Responder {
    let query = query.into_inner();
    let page_size = query.page_size
        .and_then(|size| Some(size.clamp(10, 100)))
        .unwrap_or(DEFAULT_PAGE_SIZE);

    let paginator = Book::find()
        .order_by_desc(book::Column::CreatedAt)
        .into_partial_model::<BookSchema>()
        .paginate(db.as_ref(), page_size);

    let result = paginator
        .num_pages()
        .await;
    
    let page = if let Some(page) = query.page {
        let pages_count = match result {
            Ok(count) => count,
            Err(e) => {
                tracing::error!("Failed to get page count: {:?}", e);
                return HttpResponse::InternalServerError().finish()
            },
        };

        if page >= pages_count {
            return HttpResponse::BadRequest().body(format!("Pages count = {}", pages_count))
        }

        page
    } else {
        0
    };

    let result = paginator
        .fetch_page(page)
        .await;

    let books = match result {
        Ok(books) => books,
        Err(e) => {
            tracing::error!("Failed to fetch books (page: {}): {:?}", page, e);
            return HttpResponse::InternalServerError().finish()
        }
    };

    HttpResponse::Ok().json(books)
}

pub async fn get_book(db: web::Data<DatabaseConnection>, query: web::Query<GetBookSchema>) -> impl Responder {
    let result = Book::find_by_id(query.id)
        .select_only()
        .columns([
            book::Column::Id,
            book::Column::Title,
            book::Column::Description,
            book::Column::Status,
            book::Column::Cover
        ])
        .column_as(
            Expr::cust(r#"
                COALESCE(
                    json_agg(
                        json_build_object('id', tags.id, 'name', tags.name)
                    ) FILTER (WHERE tags.id IS NOT NULL),
                    '[]'::json
                )
            "#),
            "tags"
        )
        .column_as(
            Expr::cust(r#"
                COALESCE(
                    json_agg(
                        json_build_object('id', genres.id, 'name', genres.name)
                    ) FILTER (WHERE genres.id IS NOT NULL),
                    '[]'::json
                )
            "#),
            "genres"
        )
        .column_as(
            Expr::cust(r#"
                COALESCE(
                    json_agg(
                        json_build_object('id', authors.id, 'name', authors.name)
                    ) FILTER (WHERE authors.id IS NOT NULL),
                    '[]'::json
                )
            "#),
            "authors"
        )
        .column_as(
            Expr::cust(
                "(SELECT COUNT(*) FROM chapters WHERE chapters.book_id = id)",
            ),
            "chapters_count"
        )
        .join(sea_orm::JoinType::LeftJoin, book_tag::Relation::Book.def().rev())
        .join(sea_orm::JoinType::LeftJoin, book_tag::Relation::Tag.def())
        .join(sea_orm::JoinType::LeftJoin, book_genre::Relation::Book.def().rev())
        .join(sea_orm::JoinType::LeftJoin, book_genre::Relation::Genre.def())
        .join(sea_orm::JoinType::LeftJoin, book_author::Relation::Book.def().rev())
        .join(sea_orm::JoinType::LeftJoin, book_author::Relation::Author.def())
        .group_by(book::Column::Id)
        .into_json()
        .all(db.as_ref())
        .await;
    
    match result {
        Ok(mut books) if !books.is_empty()=> {
            let book_value = books[0].take();
            let result = serde_json::from_value::<BookFullSchema>(book_value);

            match result {
                Ok(book) => HttpResponse::Ok().json(book),
                Err(e) => {
                    tracing::error!("Deserialization failed: {:?}", e);
                    HttpResponse::InternalServerError().finish()
                },
            }
        },
        Ok(_) => {
            HttpResponse::BadRequest().body("Book not found!")
        }
        Err(e) => {
            tracing::error!("Failed to select book: {:?}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn create_book(db: web::Data<DatabaseConnection>, mut payload: Multipart) -> impl Responder {
    todo!();

    HttpResponse::Ok().body("Book created!")
}

pub async fn update_book(db: web::Data<DatabaseConnection>, ) -> impl Responder {
    todo!();

    HttpResponse::Ok().body("Book updated!")
}

pub async fn search_books(search: ElasticsearchClient, query: web::Query<SearchBookSchema>) -> impl Responder {
    let result = search.search(&query.q).await;

    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            tracing::error!("Failed to search book: {:?}", e);
            // TODO: more errors
            HttpResponse::BadRequest().finish()
        },
    }
}