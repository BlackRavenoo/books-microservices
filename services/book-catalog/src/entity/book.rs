use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use serde::Serialize;

use crate::utils::validators::is_valid_url;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize)]
#[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
pub enum BookStatus {
    Ongoing = 0,
    Completed = 1,
    Hiatus = 2,
    Cancelled = 3,
}

impl BookStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BookStatus::Ongoing => "В процессе",
            BookStatus::Completed => "Завершен",
            BookStatus::Hiatus => "Заморожен",
            BookStatus::Cancelled => "Приостановлен",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "books")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub title: String,
    #[sea_orm(column_type = "String(StringLen::N(1024))")]
    pub description: String,
    #[sea_orm(default = 0)]
    pub status: BookStatus,
    pub cover: String,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(nullable)]
    pub series_id: i32
}

#[derive(Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::chapter::Entity")]
    Chapter,
    #[sea_orm(
        belongs_to = "super::series::Entity",
        from = "Column::SeriesId",
        to = "super::series::Column::Id",
        on_delete = "SetNull"
    )]
    Series
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        super::book_tag::Relation::Tag.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::book_tag::Relation::Book.def().rev())
    }
}

impl Related<super::genre::Entity> for Entity {
    fn to() -> RelationDef {
        super::book_genre::Relation::Genre.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::book_genre::Relation::Book.def().rev())
    }
}

impl Related<super::author::Entity> for Entity {
    fn to() -> RelationDef {
        super::book_author::Relation::Author.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::book_author::Relation::Book.def().rev())
    }
}

impl Related<super::chapter::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Chapter.def()
    }
}

impl Related<super::series::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Series.def()
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, _insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let url = self.cover.as_ref();
        if !is_valid_url(url) {
            return Err(DbErr::Custom("Некорректный URL обложки".into()));
        }
        Ok(self)
    }
}