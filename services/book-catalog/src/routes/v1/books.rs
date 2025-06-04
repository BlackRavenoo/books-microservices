use std::io::Read;

use actix_multipart::form::MultipartForm;
use actix_web::{web, HttpResponse, Responder};
use cache::{cache::HybridCache, expiry::Expiration, serializer::bincode::BincodeSerializer};
use sea_orm::{prelude::Expr, ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, RelationTrait, TransactionTrait};
use uuid::Uuid;
use serde_qs::actix::QsQuery;

use crate::{
    entity::{
        author, book::{self, Entity as Book}, book_author, book_genre, book_tag, chapter, series
    }, schema::{BookFullSchema, BookSchema, CreateBookForm, GetBookSchema, GetListSchema, PaginationSchema, UpdateBookForm}, storage::{s3::S3StorageBackend, StorageId}, utils::{db::{insert_book_relations, remove_book_relations}, image::process_image}
};

const DEFAULT_PAGE_SIZE: u64 = 50;

pub async fn get_books(db: web::Data<DatabaseConnection>, query: QsQuery<GetListSchema>) -> impl Responder {
    let query = query.into_inner();
    let page_size = query.page_size
        .and_then(|size| Some(size.clamp(10, 100)))
        .unwrap_or(DEFAULT_PAGE_SIZE);

    let order_by = query.order_by.unwrap_or(crate::schema::OrderBy::CreatedAt);

    let select = Book::find();

    let mut paginator = match order_by {
        crate::schema::OrderBy::ChaptersCount => select
            .join(sea_orm::JoinType::LeftJoin, book::Relation::Chapter.def())
            .group_by(book::Column::Id)
            .order_by_desc(Expr::count(chapter::Column::Id.into_expr())),
        crate::schema::OrderBy::CreatedAt => select.order_by_desc(book::Column::CreatedAt),
        crate::schema::OrderBy::NameDesc => select.order_by_desc(book::Column::Title),
        crate::schema::OrderBy::NameAsc => select.order_by_asc(book::Column::Title),
    };

    if let (Some(target), Some(target_id)) = (query.target, query.target_id) {
        match target {
            crate::schema::Target::Author => {
                paginator = paginator
                    .join(sea_orm::JoinType::LeftJoin, book_author::Relation::Book.def().rev())
                    .join(sea_orm::JoinType::InnerJoin, book_author::Relation::Author.def())
                    .filter(author::Column::Id.eq(target_id))
            },
            crate::schema::Target::Series => {
                paginator = paginator
                    .join(sea_orm::JoinType::InnerJoin, book::Relation::Series.def())
                    .filter(series::Column::Id.eq(target_id))
            },
        }
    }

    if let Some(genres) = query.genres_include {
        if !genres.is_empty() {
            let count = genres.len() as i64;
            
            let subquery = book_genre::Entity::find()
                .filter(book_genre::Column::GenreId.is_in(genres))
                .group_by(book_genre::Column::BookId)
                .having(Expr::expr(Expr::col(book_genre::Column::GenreId).count_distinct()).eq(count))
                .select_only()
                .column(book_genre::Column::BookId)
                .into_query();
            
            paginator = paginator.filter(book::Column::Id.in_subquery(subquery));
        }
    }

    if let Some(genres) = query.genres_exclude {
        if !genres.is_empty() {
            let subquery = book_genre::Entity::find()
                .filter(Expr::col(book_genre::Column::BookId).equals(book::Column::Id))
                .filter(book_genre::Column::GenreId.is_in(genres))
                .select_only()
                .column(book_genre::Column::BookId)
                .into_query();
            
            paginator = paginator.filter(Expr::exists(subquery).not());
        }
    }

    if let Some(tags) = query.tags_include {
        if !tags.is_empty() {
            let count = tags.len() as i64;
            
            let subquery = book_tag::Entity::find()
                .filter(book_tag::Column::TagId.is_in(tags))
                .group_by(book_tag::Column::BookId)
                .having(Expr::expr(Expr::col(book_tag::Column::TagId).count_distinct()).eq(count))
                .select_only()
                .column(book_tag::Column::BookId)
                .into_query();
            
            paginator = paginator.filter(book::Column::Id.in_subquery(subquery));
        }
    }

    if let Some(tags) = query.tags_exclude {
        if !tags.is_empty() {
            let subquery = book_tag::Entity::find()
                .filter(Expr::col(book_tag::Column::BookId).equals(book::Column::Id))
                .filter(book_tag::Column::TagId.is_in(tags))
                .select_only()
                .column(book_tag::Column::BookId)
                .into_query();
            
            paginator = paginator.filter(Expr::exists(subquery).not());
        }
    }

    if let Some(statuses) = query.statuses {
        if !statuses.is_empty() {
            paginator = paginator.filter(book::Column::Status.is_in(statuses));
        }
    }

    let paginator = paginator
        .into_partial_model::<BookSchema>()
        .paginate(db.as_ref(), page_size);

    let result = paginator
        .num_items_and_pages()
        .await;
    
    let total = match result {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to get page count: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        },
    };

    let page = if let Some(page) = query.page {
        if page > total.number_of_pages {
            return HttpResponse::Ok().json(PaginationSchema{
                max_page: total.number_of_pages,
                total_items: total.number_of_items,
                items: Vec::<BookSchema>::new(),
            })
        }

        page - 1
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

    let resp = PaginationSchema {
        max_page: total.number_of_pages,
        total_items: total.number_of_items,
        items: books,
    };

    HttpResponse::Ok().json(resp)
}

pub async fn get_book(
    db: web::Data<DatabaseConnection>,
    cache: web::Data<HybridCache<String, BookFullSchema, BincodeSerializer<BookFullSchema>>>,
    query: web::Path<GetBookSchema>,
) -> impl Responder {
    match cache.get(
        &query.id.to_string(),
        Expiration::Minutes(10)
    ).await {
        Ok(Some(book)) => return HttpResponse::Ok().json(book),
        Ok(None) => (),
        Err(e) => tracing::error!("Failed to get book from cache: {:?}", e),
    };

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
                        DISTINCT jsonb_build_object('id', tags.id, 'name', tags.name)
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
                        DISTINCT jsonb_build_object('id', genres.id, 'name', genres.name)
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
                        DISTINCT jsonb_build_object('id', authors.id, 'name', authors.name)
                    ) FILTER (WHERE authors.id IS NOT NULL),
                    '[]'::json
                )
            "#),
            "authors"
        )
        .column_as(
            Expr::cust(
                "(SELECT COUNT(*) FROM chapters WHERE chapters.book_id = books.id)",
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
                Ok(book) => {
                    if let Err(e) = cache.set(
                        query.id.to_string(),
                        book.clone(),
                        Expiration::Minutes(10)
                    ).await {
                        tracing::error!("Failed to insert book into cache: {:?}", e);
                    };
                    HttpResponse::Ok().json(book)
                },
                Err(e) => {
                    tracing::error!("Deserialization failed: {:?}", e);
                    HttpResponse::InternalServerError().finish()
                },
            }
        },
        Ok(_) => {
            HttpResponse::NotFound().body("Book not found!")
        }
        Err(e) => {
            tracing::error!("Failed to select book: {:?}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn create_book(
    db: web::Data<DatabaseConnection>,
    storage: web::Data<S3StorageBackend>,
    MultipartForm(form): MultipartForm<CreateBookForm>
) -> impl Responder {
    let mut cover = form.cover;
    let fields = form.fields.into_inner();
    let storage_id = StorageId::BookCover as u32;

    let mut buf = Vec::new();

    if let Err(e) = cover.file.read_to_end(&mut buf) {
        tracing::error!("Failed to read uploaded cover file: {:?}", e);
        return HttpResponse::BadRequest().body("Could not read uploaded file");
    }

    let image = match process_image(&buf, 375) {
        Ok(image) => image,
        Err(e) => {
            tracing::error!("Failed to process image: {:?}", e);
            return HttpResponse::BadRequest().body("Could not process uploaded image")
        },
    };

    let id = Uuid::new_v4();

    let url = storage.get_image_url(storage_id, id);

    let book = book::ActiveModel {
        title: Set(fields.title),
        description: Set(fields.description),
        status: Set(fields.status),
        cover: Set(url),
        series_id: Set(fields.series_id),
        ..Default::default()
    };

    let transaction = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        },
    };

    let book_id = match book.insert(&transaction).await {
        Ok(model) => model.id,
        Err(e) => {
            tracing::error!("Failed to insert book: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        },
    };

    if !fields.tags.is_empty() {
        if let Err(e) = insert_book_relations::<_, book_tag::Entity, _>(
            &transaction,
            book_id,
            fields.tags,
            |book_id, tag_id| book_tag::ActiveModel {
                book_id: Set(book_id),
                tag_id: Set(tag_id),
            }
        ).await {
            tracing::error!("Failed to insert books_tags: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    }

    if !fields.genres.is_empty() {
        if let Err(e) = insert_book_relations::<_, book_genre::Entity, _>(
            &transaction,
            book_id,
            fields.genres,
            |book_id, genre_id| book_genre::ActiveModel {
                book_id: Set(book_id),
                genre_id: Set(genre_id),
            }
        ).await {
            tracing::error!("Failed to insert books_genres: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    }

    if !fields.authors.is_empty() {
        if let Err(e) = insert_book_relations::<_, book_author::Entity, _>(
            &transaction,
            book_id,
            fields.authors,
            |book_id, author_id| book_author::ActiveModel {
                book_id: Set(book_id),
                author_id: Set(author_id),
            }
        ).await {
            tracing::error!("Failed to insert books_authors: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    }

    match storage.save(storage_id, id, image, "image/jpeg", "jpg").await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Failed to upload cover {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    };

    match transaction.commit().await {
        Ok(_) => HttpResponse::Ok().body("Book created!"),
        Err(e) => {
            tracing::error!("Failed to commit transaction: {:?}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn update_book(
    db: web::Data<DatabaseConnection>,
    storage: web::Data<S3StorageBackend>,
    book_id: web::Path<i32>,
    cache: web::Data<HybridCache<String, BookFullSchema, BincodeSerializer<BookFullSchema>>>,
    MultipartForm(form): MultipartForm<UpdateBookForm>
) -> impl Responder {
    let cover = form.cover;
    let book_id = book_id.into_inner();
    let fields = form.fields.into_inner();

    let transaction = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        },
    };

    let book = match book::Entity::find_by_id(book_id)
        .one(&transaction)
        .await {
            Ok(Some(b)) => b,
            Ok(None) => {
                return HttpResponse::NotFound().body("Book not found");
            },
            Err(e) => {
                tracing::error!("Failed to find book: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

    let cover_id = match storage.extract_uuid_from_url(&book.cover) {
        Some(uuid) => uuid,
        None => {
            tracing::error!("Failed to get uuid from url.");
            return HttpResponse::InternalServerError().finish()
        },
    };

    let mut book_active: book::ActiveModel = book.into();

    if let Some(title) = fields.title {
        book_active.title = Set(title);
    }
    
    if let Some(description) = fields.description {
        book_active.description = Set(description);
    }
    
    if let Some(status) = fields.status {
        book_active.status = Set(status);
    }
    
    if let Some(series_id) = fields.series_id {
        book_active.series_id = Set(Some(series_id));
    }

    if let Some(mut cover) = cover {
        let mut buf = Vec::new();

        if let Err(e) = cover.file.read_to_end(&mut buf) {
            tracing::error!("Failed to read uploaded cover file: {:?}", e);
            return HttpResponse::BadRequest().body("Could not read uploaded file");
        }

        let image = match process_image(&buf, 375) {
            Ok(image) => image,
            Err(e) => {
                tracing::error!("Failed to process image: {:?}", e);
                return HttpResponse::BadRequest().body("Could not process uploaded image")
            },
        };

        if storage.save(StorageId::BookCover as u32, cover_id, image, "image/jpeg", "jpg").await.is_err() {
            return HttpResponse::InternalServerError().finish()
        };
    }

    if let Err(e) = book_active.update(&transaction).await {
        tracing::error!("Failed to update book: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    if !fields.tags_to_add.is_empty() {
        if let Err(e) = insert_book_relations::<_, book_tag::Entity, _>(
            &transaction,
            book_id,
            fields.tags_to_add,
            |book_id, tag_id| book_tag::ActiveModel {
                book_id: Set(book_id),
                tag_id: Set(tag_id),
            }
        ).await {
            tracing::error!("Failed to insert books_tags: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    }

    if !fields.tags_to_delete.is_empty() {
        if let Err(e) = remove_book_relations::<_, book_tag::Entity, _>(
            &transaction,
            book_id,
            fields.tags_to_delete,
            book_tag::Column::BookId,
            book_tag::Column::TagId
        ).await {
            tracing::error!("Failed to remove books_tags: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    }

    if !fields.genres_to_add.is_empty() {
        if let Err(e) = insert_book_relations::<_, book_genre::Entity, _>(
            &transaction,
            book_id,
            fields.genres_to_add,
            |book_id, genre_id| book_genre::ActiveModel {
                book_id: Set(book_id),
                genre_id: Set(genre_id),
            }
        ).await {
            tracing::error!("Failed to insert books_genres: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    }

    if !fields.genres_to_delete.is_empty() {
        if let Err(e) = remove_book_relations::<_, book_genre::Entity, _>(
            &transaction,
            book_id,
            fields.genres_to_delete,
            book_genre::Column::BookId,
            book_genre::Column::GenreId
        ).await {
            tracing::error!("Failed to remove books_genres: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    }

    if !fields.authors_to_add.is_empty() {
        if let Err(e) = insert_book_relations::<_, book_author::Entity, _>(
            &transaction,
            book_id,
            fields.authors_to_add,
            |book_id, author_id| book_author::ActiveModel {
                book_id: Set(book_id),
                author_id: Set(author_id),
            }
        ).await {
            tracing::error!("Failed to insert books_author: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    }

    if !fields.authors_to_delete.is_empty() {
        if let Err(e) = remove_book_relations::<_, book_author::Entity, _>(
            &transaction,
            book_id,
            fields.authors_to_delete,
            book_author::Column::BookId,
            book_author::Column::AuthorId
        ).await {
            tracing::error!("Failed to remove books_authors: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    }

    match transaction.commit().await {
        Ok(_) => {
            let _ = cache.invalidate(book_id.to_string()).await;
            HttpResponse::Ok().body("Book updated!")
        },
        Err(e) => {
            tracing::error!("Failed to commit transaction: {:?}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}