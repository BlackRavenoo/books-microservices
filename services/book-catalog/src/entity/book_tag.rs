use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "books_tags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    book_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    tag_id: i16,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::book::Entity",
        from = "Column::BookId",
        to = "super::book::Column::Id",
        on_delete = "Cascade"
    )]
    Book,
    #[sea_orm(
        belongs_to = "super::tag::Entity",
        from = "Column::TagId",
        to = "super::tag::Column::Id",
        on_delete = "Restrict"
    )]
    Tag,
}

impl ActiveModelBehavior for ActiveModel {}