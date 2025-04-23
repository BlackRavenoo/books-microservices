use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter};

pub async fn insert_book_relations<C, E, I>(
    db: &C,
    book_id: i32,
    relation_ids: Vec<I>,
    create_model: impl Fn(i32, I) -> E::ActiveModel
) -> Result<(), DbErr>
where
    E: EntityTrait,
    C: ConnectionTrait
{
    let models = relation_ids
        .into_iter()
        .map(|id| create_model(book_id, id))
        .collect::<Vec<_>>();
    
    match E::insert_many(models).exec(db).await {
        Ok(_) => Ok(()),
        Err(e) => {
            Err(e)
        }
    }
}

pub async fn remove_book_relations<C, E, I>(
    transaction: &C,
    book_id: i32,
    relation_ids: Vec<I>,
    book_id_column: E::Column,
    relation_id_column: E::Column,
) -> Result<(), DbErr>
where
    E: EntityTrait,
    C: ConnectionTrait,
    I: Into<sea_orm::Value>
{
    if relation_ids.is_empty() {
        return Ok(());
    }
    
    E::delete_many()
        .filter(book_id_column.eq(book_id))
        .filter(relation_id_column.is_in(relation_ids))
        .exec(transaction)
        .await?;
        
    Ok(())
}