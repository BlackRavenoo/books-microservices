use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "books_authors")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    book_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    author_id: i32,
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
        belongs_to = "super::author::Entity",
        from = "Column::AuthorId",
        to = "super::author::Column::Id",
        on_delete = "Restrict"
    )]
    Author,
}

impl ActiveModelBehavior for ActiveModel {}