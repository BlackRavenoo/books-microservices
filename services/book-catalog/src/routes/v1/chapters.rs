use actix_web::{web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait};
use uuid::Uuid;

use crate::{entity::chapter, schema::{ChapterFullSchema, CreateChapterFields, InputChapterSchema, UpdateChapterFields}, storage::{s3::S3StorageBackend, StorageId}};

pub async fn create_chapter(
    db: web::Data<DatabaseConnection>,
    storage: web::Data<S3StorageBackend>,
    book_id: web::Path<i32>,
    form: web::Json<CreateChapterFields>
) -> impl Responder {
    let book_id = book_id.into_inner();
    let fields = form.into_inner();

    let transaction = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        },
    };

    let object_id = Uuid::new_v4();
    let storage_id = StorageId::ChapterContent as u32;

    let json_string = match serde_json::to_string_pretty(&fields.content) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to get pretty string: {:?}", e);
            return  HttpResponse::BadRequest().finish();
        },
    };
    let data = json_string.into_bytes();

    let key = format!("{}/{}.json", storage_id, object_id);

    if let Err(e) = storage.save_with_custom_key(&key, "application/json", data).await {
        tracing::error!("Failed to save chapter content: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    };

    let chapter = chapter::ActiveModel {
        index: Set(fields.index),
        book_id: Set(book_id),
        name: Set(fields.name),
        key: Set(key.clone()),
        ..Default::default()
    };

    let chapter_id = match chapter.insert(&transaction).await {
        Ok(chapter) => chapter.id,
        Err(e) => {
            tracing::error!("Failed to insert chapter: {:?}", e);
            let _ = storage.delete_by_key(&key).await;
            return HttpResponse::InternalServerError().finish();
        },
    };

    match transaction.commit().await {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "id": chapter_id,
            "message": "Chapter created!"
        })),
        Err(e) => {
            tracing::error!("Failed to commit transaction: {:?}", e);
            if let Err(e) = storage.delete_by_key(&key).await {
                tracing::error!("Failed to delete chapter content after failed transaction: {:?}", e)
            };
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn get_chapter(
    db: web::Data<DatabaseConnection>,
    storage: web::Data<S3StorageBackend>,
    path: web::Path<i32>,
    query: web::Query<InputChapterSchema>
) -> impl Responder {
    let book_id = path.into_inner();
    let chapter_number = query.number;

    let chapter = match chapter::Entity::find()
        .filter(chapter::Column::BookId.eq(book_id))
        .filter(chapter::Column::Index.eq(chapter_number))
        .one(db.as_ref())
        .await {
            Ok(Some(chapter)) => chapter,
            Ok(None) => return HttpResponse::NotFound().finish(),
            Err(e) => {
                tracing::error!("Failed to get chapter: {:?}", e);
                return HttpResponse::InternalServerError().finish()
            }
        };

    let content_bytes = match storage.get_by_key(&chapter.key).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to get chapter content: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        },
    };

    let content = match String::from_utf8(content_bytes) {
        Ok(json_str) => {
            match serde_json::from_str::<serde_json::Value>(&json_str) {
                Ok(content) => content,
                Err(e) => {
                    tracing::error!("Failed to parse chapter content JSON: {:?}", e);
                    return HttpResponse::InternalServerError().finish();
                }
            }
        },
        Err(e) => {
            tracing::error!("Failed to convert content to string: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let response = ChapterFullSchema {
        id: chapter.id,
        index: chapter.index,
        name: chapter.name,
        content: Some(content),
        book_id: chapter.book_id,
        created_at: chapter.created_at,
    };

    HttpResponse::Ok().json(response)
}

pub async fn update_chapter(
    db: web::Data<DatabaseConnection>,
    storage: web::Data<S3StorageBackend>,
    path: web::Path<i32>,
    form: web::Json<UpdateChapterFields>,
    query: web::Query<InputChapterSchema>
) -> impl Responder {
    let book_id = path.into_inner();
    let chapter_number = query.number;
    let fields = form.into_inner();

    let transaction = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        },
    };

    let chapter = match chapter::Entity::find()
        .filter(chapter::Column::BookId.eq(book_id))
        .filter(chapter::Column::Index.eq(chapter_number))
        .one(&transaction)
        .await {
            Ok(Some(chapter)) => chapter,
            Ok(None) => return HttpResponse::NotFound().finish(),
            Err(e) => {
                tracing::error!("Failed to get chapter: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

    let cur_index = chapter.index;
    let content_key = chapter.key.clone();

    let mut chapter_active: chapter::ActiveModel = chapter.into();

    if let Some(content) = fields.content {
        let json_string = match serde_json::to_string_pretty(&content) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to serialize content: {:?}", e);
                return HttpResponse::BadRequest().finish()
            },
        };
        let data = json_string.into_bytes();

        if let Err(e) = storage.save_with_custom_key(&content_key, "application/json", data).await {
            tracing::error!("Failed to update chapter content in S3: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    }

    if let Some(name) = fields.name {
        chapter_active.name = Set(name);
    }

    if let Some(new_index) = fields.index {
        if new_index != cur_index {
            let existing_chapter = chapter::Entity::find()
                .filter(chapter::Column::BookId.eq(book_id))
                .filter(chapter::Column::Index.eq(new_index))
                .select_only()
                .column(chapter::Column::Id)
                .one(&transaction)
                .await;
                
            match existing_chapter {
                Ok(Some(_)) => {
                    return HttpResponse::Conflict().finish()
                },
                Ok(None) => {
                    chapter_active.index = Set(new_index);
                },
                Err(e) => {
                    tracing::error!("Failed to check existing chapter: {:?}", e);
                    return HttpResponse::InternalServerError().finish()
                }
            }
        }
    }

    if let Err(e) = chapter_active.update(&transaction).await {
        tracing::error!("Failed to update chapter in database: {:?}", e);
        return HttpResponse::InternalServerError().finish()
    }

    match transaction.commit().await {
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            tracing::error!("Failed to commit transaction: {:?}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn delete_chapter(
    db: web::Data<DatabaseConnection>,
    storage: web::Data<S3StorageBackend>,
    book_id: web::Path<i32>,
    query: web::Query<InputChapterSchema>
) -> impl Responder {
    let book_id = book_id.into_inner();
    let chapter_index = query.number;

    let transaction = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        },
    };

    let chapter = match chapter::Entity::delete_many()
        .filter(chapter::Column::BookId.eq(book_id))
        .filter(chapter::Column::Index.eq(chapter_index))
        .exec_with_returning(&transaction)
        .await {
            Ok(mut chapters) if !chapters.is_empty() => chapters.remove(0),
            Ok(_) => return HttpResponse::NotFound().finish(),
            Err(e) => {
                tracing::error!("Failed to delete chapter: {:?}", e);
                return HttpResponse::InternalServerError().finish()
            }
        };

    if let Err(e) = storage.delete_by_key(&chapter.key).await {
        tracing::error!("Failed to delete chapter content from S3: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    match transaction.commit().await {
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            tracing::error!("Failed to commit transaction: {:?}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn get_chapters(
    db: web::Data<DatabaseConnection>,
    book_id: web::Path<i32>
) -> impl Responder {
    let book_id = book_id.into_inner();

    let chapters = match chapter::Entity::find()
        .filter(chapter::Column::BookId.eq(book_id))
        .order_by_asc(chapter::Column::Index)
        .into_partial_model::<ChapterFullSchema>()
        .all(db.as_ref())
        .await {
            Ok(chapters) => chapters,
            Err(e) => {
                tracing::error!("Failed to get chapters: {:?}", e);
                return HttpResponse::InternalServerError().finish()
            }
        };

    HttpResponse::Ok().json(chapters)
}