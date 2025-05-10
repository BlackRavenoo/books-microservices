use std::io::Read;

use actix_multipart::form::MultipartForm;
use actix_web::{web, HttpResponse, Responder};
use cache::{cache::HybridCache, expiry::Expiration};
use sea_orm::{prelude::Expr, ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, Iterable, PaginatorTrait, QueryOrder, QuerySelect, RelationTrait, TransactionTrait};
use uuid::Uuid;

use crate::{
    entity::{
        book::{self, BookStatus, Entity as Book}, book_author, book_genre, book_tag, chapter, genre, tag
    }, schema::{BookFullSchema, BookSchema, ConstantsSchema, CreateBookForm, Genre, GetBookSchema, GetListSchema, Tag, UpdateBookForm}, storage::s3::S3StorageBackend, utils::{db::{insert_book_relations, remove_book_relations}, image::process_image}
};

const DEFAULT_PAGE_SIZE: u64 = 50;

pub enum StorageId {
    Cover = 0
}

pub async fn get_books(db: web::Data<DatabaseConnection>, query: web::Query<GetListSchema>) -> impl Responder {
    let query = query.into_inner();
    let page_size = query.page_size
        .and_then(|size| Some(size.clamp(10, 100)))
        .unwrap_or(DEFAULT_PAGE_SIZE);

    let order_by = query.order_by.unwrap_or(crate::schema::OrderBy::CreatedAt);

    let select = Book::find();

    let paginator = match order_by {
        crate::schema::OrderBy::ChaptersCount => select
            .join(sea_orm::JoinType::LeftJoin, book::Relation::Chapter.def())
            .group_by(book::Column::Id)
            .order_by_desc(Expr::count(chapter::Column::Id.into_expr())),
        crate::schema::OrderBy::CreatedAt => select.order_by_desc(book::Column::CreatedAt),
        crate::schema::OrderBy::NameDesc => select.order_by_desc(book::Column::Title),
        crate::schema::OrderBy::NameAsc => select.order_by_asc(book::Column::Title),
    };

    let paginator = paginator
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

pub async fn get_book(
    db: web::Data<DatabaseConnection>,
    cache: web::Data<HybridCache<String, BookFullSchema>>,
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
            HttpResponse::BadRequest().body("Book not found!")
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
    let storage_id = StorageId::Cover as u32;

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

    let url = storage.get_url(storage_id, id);

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
            let _ = storage.delete(storage_id, id).await;
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

    match storage.save(storage_id, id, image).await {
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
    cache: web::Data<HybridCache<String, BookFullSchema>>,
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

        if storage.save(StorageId::Cover as u32, cover_id, image).await.is_err() {
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

pub async fn get_constants(
    db: web::Data<DatabaseConnection>,
    cache: web::Data<HybridCache<String, ConstantsSchema>>
) -> impl Responder {
    match cache.get(&String::new(), Expiration::Minutes(10)).await {
        Ok(Some(consts)) => return HttpResponse::Ok().json(consts),
        Ok(None) => (),
        Err(e) => tracing::error!("Failed to get constants from cache: {:?}", e),
    };

    let tags = match tag::Entity::find()
        .into_partial_model::<Tag>()
        .all(db.as_ref())
        .await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to fetch tags from db: {:?}", e);
                return HttpResponse::InternalServerError().finish()
            },
        };

    let genres = match genre::Entity::find()
        .into_partial_model::<Genre>()
        .all(db.as_ref())
        .await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to fetch genres from db: {:?}", e);
                return HttpResponse::InternalServerError().finish()
            },
        };

    let constants = ConstantsSchema {
        tags,
        genres,
        status: BookStatus::iter().collect()
    };

    if let Err(e) = cache.set(String::new(), constants.clone(), Expiration::Minutes(10)).await {
        tracing::error!("Failed to insert constants into cache: {:?}", e);
    };

    HttpResponse::Ok().json(constants)
}