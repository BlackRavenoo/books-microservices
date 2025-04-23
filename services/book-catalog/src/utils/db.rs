use sea_orm::{ConnectionTrait, DbErr, EntityTrait};

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