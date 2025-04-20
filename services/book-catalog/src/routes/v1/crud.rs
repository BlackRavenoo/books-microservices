use actix_web::{web, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect};

use crate::{entity::book::{self, Entity as Book}, schema::{BookSchema, GetBookSchema, GetListSchema}};

const DEFAULT_PAGE_SIZE: u64 = 50;

// TODO: add custom order_by and custom fields
pub async fn get_books(db: web::Data<DatabaseConnection>, query: web::Query::<GetListSchema>) -> impl Responder {
    let query = query.into_inner();
    let page_size = query.page_size
        .and_then(|size| Some(size.clamp(10, 100)))
        .unwrap_or(DEFAULT_PAGE_SIZE);

    let paginator = Book::find()
        .select_only()
        .column(book::Column::Id)
        .column(book::Column::Title)
        .column(book::Column::Cover)
        .order_by_desc(book::Column::CreatedAt)
        .into_tuple()
        .paginate(db.as_ref(), page_size);

    let result = paginator
        .num_pages()
        .await;
    
    let page = if let Some(page) = query.page {
        let pages_count = if let Err(e) = result {
            tracing::error!("Failed to get page count: {:#?}", e);
            return HttpResponse::InternalServerError().finish()
        } else {
            result.unwrap()
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

    let books = if let Err(e) = result {
        tracing::error!("Failed to fetch books (page: {}): {:#?}", page, e);
        return HttpResponse::InternalServerError().finish()
    } else {
        result.unwrap().into_iter().map(|(id, title, cover)| BookSchema {
            id,
            title,
            thumbnail: cover,
        })
        .collect::<Vec<_>>()
    };

    HttpResponse::Ok().json(books)
}

pub async fn get_book(db: web::Data<DatabaseConnection>, query: web::Query::<GetBookSchema>) -> impl Responder {
    let result = Book::find_by_id(query.id)
        .one(db.as_ref())
        .await;

    if let Err(e) = result {
        tracing::error!("Failed to select book: {:#?}", e);
        HttpResponse::InternalServerError().finish()
    } else {
        let book = result.unwrap();
        if let Some(book) = book {
            HttpResponse::Ok().json(book)
        } else {
            HttpResponse::BadRequest().body("Book does not exist.")
        }
    }
}

