use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "chapters")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub index: i16,
    pub book_id: i32,
    pub name: String,
    pub link: String,
    pub created_at: DateTimeWithTimeZone
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::book::Entity",
        from = "Column::BookId",
        to = "super::book::Column::Id"
    )]
    Book
}

impl Related<super::book::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Book.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}