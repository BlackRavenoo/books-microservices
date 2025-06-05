use std::io::Read;

use actix_multipart::form::MultipartForm;
use actix_web::{web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DerivePartialModel, EntityTrait, FromQueryResult, QuerySelect, TransactionTrait};
use uuid::Uuid;

use crate::{
    entity::author, schema::{CreateAuthorForm, UpdateAuthorForm}, storage::{s3::S3StorageBackend, StorageId}, utils::image::process_image
};

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "author::Entity")]
struct AuthorCover {
    cover: String
}

pub async fn get_author(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> impl Responder {
    let author_id = path.into_inner();
    
    let result = author::Entity::find_by_id(author_id)
        .one(db.as_ref())
        .await;
    
    match result {
        Ok(Some(author)) => {
            HttpResponse::Ok().json(author)
        },
        Ok(None) => {
            HttpResponse::NotFound().body("Author not found")
        },
        Err(e) => {
            tracing::error!("Failed to fetch author: {:?}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn update_author(
    db: web::Data<DatabaseConnection>,
    storage: web::Data<S3StorageBackend>,
    author_id: web::Path<i32>,
    MultipartForm(form): MultipartForm<UpdateAuthorForm>
) -> impl Responder {
    let cover = form.cover;
    let name = form.fields.0.name;
    let author_id = author_id.into_inner();

    let transaction = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        },
    };

    let author = match author::Entity::find_by_id(author_id)
        .one(&transaction)
        .await {
            Ok(Some(a)) => a,
            Ok(None) => {
                return HttpResponse::NotFound().body("Author not found");
            },
            Err(e) => {
                tracing::error!("Failed to find author: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

    let cover_id = match storage.extract_uuid_from_url(&author.cover) {
        Some(uuid) => uuid,
        None => {
            tracing::error!("Failed to get uuid from url.");
            return HttpResponse::InternalServerError().finish()
        },
    };

    let mut author_active: author::ActiveModel = author.into();

    if let Some(name) = name {
        author_active.name = Set(name);
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

        if storage.save(StorageId::AuthorCover as u32, cover_id, image, "image/jpeg", "jpg").await.is_err() {
            return HttpResponse::InternalServerError().finish()
        };
    }

    if let Err(e) = author_active.update(&transaction).await {
        tracing::error!("Failed to update author: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    match transaction.commit().await {
        Ok(_) => {
            HttpResponse::Ok().body("Author updated!")
        },
        Err(e) => {
            tracing::error!("Failed to commit transaction: {:?}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn create_author(
    db: web::Data<DatabaseConnection>,
    storage: web::Data<S3StorageBackend>,
    MultipartForm(form): MultipartForm<CreateAuthorForm>
) -> impl Responder {
    let cover = form.cover;
    let name = form.fields.0.name;
    let storage_id = StorageId::AuthorCover as u32;
    let id = Uuid::new_v4();

    let (image, url) = match cover {
        Some(mut cover) => {
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
        
            let url = storage.get_image_url(storage_id, id);

            (Some(image), url)
        },
        None => (None, storage.get_placeholder_url(storage_id)),
    };


    let author = author::ActiveModel {
        name: Set(name),
        cover: Set(url),
        ..Default::default()
    };

    let transaction = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        },
    };
    
    let author_id = match author.insert(&transaction).await {
        Ok(author) => author.id,
        Err(e) => {
            tracing::error!("Failed to insert book: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        },
    };

    if let Some(image) = image {
        match storage.save(storage_id, id, image, "image/jpeg", "jpg").await {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Failed to upload cover {:?}", e);
                return HttpResponse::InternalServerError().finish()
            }
        };
    }

    match transaction.commit().await {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "id": author_id
        })),
        Err(e) => {
            tracing::error!("Failed to commit transaction: {:?}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn delete_author(
    db: web::Data<DatabaseConnection>,
    storage: web::Data<S3StorageBackend>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();

    let author = author::Entity::find_by_id(id)
        .select_only()
        .column(author::Column::Cover)
        .into_partial_model::<AuthorCover>()
        .one(db.as_ref())
        .await;

    match author {
        Ok(Some(author)) => {
            let cover = author.cover;
            if let Err(e) = author::Entity::delete_by_id(id)
                .exec(db.as_ref()).await {
                    tracing::error!("Failed to delete author: {:?}", e);
                    return HttpResponse::InternalServerError().finish();
                };

            if cover != storage.get_placeholder_url(StorageId::AuthorCover as u32) {
                match storage.delete_by_url(&cover).await {
                    Ok(_) => (),
                    Err(e) => {
                        tracing::error!("Failed to delete author cover: {:?}", e)
                    },
                };
            };
        

            HttpResponse::Ok().finish()
        }
        Ok(None) => {
            HttpResponse::NotFound().body("Author not found")
        }
        Err(e) => {
            tracing::warn!("Failed to get author: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}