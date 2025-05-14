use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "authors")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(column_type = "String(StringLen::N(256))")]
    pub cover: String
}

#[derive(Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::book::Entity> for Entity {
    fn to() -> RelationDef {
        super::book_author::Relation::Book.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::book_author::Relation::Author.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}