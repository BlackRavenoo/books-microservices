use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i16,
    #[sea_orm(column_type = "String(StringLen::N(64))")]
    pub name: String
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::book::Entity> for Entity {
    fn to() -> RelationDef {
        super::book_tag::Relation::Book.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::book_tag::Relation::Tag.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}