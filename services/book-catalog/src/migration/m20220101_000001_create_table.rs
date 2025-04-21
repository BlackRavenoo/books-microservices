use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
pub enum Book {
    #[sea_orm(iden = "books")]
    Table,
    Id,
    Title,
    Description,
    Status,
    Cover,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum Tag {
    #[sea_orm(iden = "tags")]
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
pub enum Genre {
    #[sea_orm(iden = "genres")]
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
pub enum Series {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
pub enum Chapter {
    #[sea_orm(iden = "chapters")]
    Table,
    Id,
    Index,
    BookId,
    Name,
    Link,
    CreatedAt
}

#[derive(DeriveIden)]
pub enum Author {
    #[sea_orm(iden = "authors")]
    Table,
    Id,
    Name,
    Cover,
}

#[derive(DeriveIden)]
pub enum BookTag {
    #[sea_orm(iden = "books_tags")]
    Table,
    BookId,
    TagId,
}

#[derive(DeriveIden)]
pub enum BookGenre {
    #[sea_orm(iden = "books_genres")]
    Table,
    BookId,
    GenreId,
}

#[derive(DeriveIden)]
pub enum BookAuthor {
    #[sea_orm(iden = "books_authors")]
    Table,
    BookId,
    AuthorId,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Book::Table)
                .if_not_exists()
                .col(pk_auto(Book::Id).integer())
                .col(string(Book::Title).string_len(255).not_null())
                .col(string(Book::Description).string_len(1024))
                .col(tiny_integer(Book::Status).not_null().default(0))
                .col(string(Book::Cover).not_null())
                .col(timestamp_with_time_zone(Book::CreatedAt).not_null().default(Expr::current_timestamp()))
                .to_owned()
        )
        .await?;

        manager.create_table(
            Table::create()
                .table(Tag::Table)
                .col(pk_auto(Tag::Id).small_integer())
                .col(string(Tag::Name).string_len(64).not_null())
                .to_owned()
        )
        .await?;

        manager.create_table(
            Table::create()
                .table(Genre::Table)
                .col(pk_auto(Genre::Id).small_integer())
                .col(string(Genre::Name).string_len(64).not_null())
                .to_owned()
        )
        .await?;

        manager.create_table(
            Table::create()
                .table(Series::Table)
                .col(pk_auto(Series::Id).integer())
                .col(string(Series::Name).not_null())
                .to_owned()
        )
        .await?;

        manager.create_table(
            Table::create()
                .table(Chapter::Table)
                .col(big_integer(Chapter::Id))
                .col(small_integer(Chapter::Index))
                .col(integer(Chapter::BookId))
                .col(string(Chapter::Name).not_null())
                .col(string(Chapter::Link).not_null())
                .col(timestamp_with_time_zone(Chapter::CreatedAt).not_null().default(Expr::current_timestamp()))
                .to_owned()
        )
        .await?;

        manager.create_table(
            Table::create()
                .table(Author::Table)
                .col(pk_auto(Author::Id).integer())
                .col(string(Author::Name))
                .col(string(Author::Cover).string_len(256))
                .to_owned()
        )
        .await?;

        manager.create_table(
            Table::create()
                .table(BookGenre::Table)
                .col(integer(BookGenre::BookId).not_null())
                .col(small_integer(BookGenre::GenreId).not_null())
                .primary_key(
                    Index::create()
                        .col(BookGenre::BookId)
                        .col(BookGenre::GenreId)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(BookGenre::Table, BookGenre::BookId)
                        .to(Book::Table, Book::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(BookGenre::Table, BookGenre::GenreId)
                        .to(Genre::Table, Genre::Id)
                        .on_delete(ForeignKeyAction::Restrict)
                )
                .to_owned()
        )
        .await?;

        manager.create_table(
            Table::create()
                .table(BookAuthor::Table)
                .col(integer(BookAuthor::BookId).not_null())
                .col(small_integer(BookAuthor::AuthorId).not_null())
                .primary_key(
                    Index::create()
                        .col(BookAuthor::BookId)
                        .col(BookAuthor::AuthorId)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(BookAuthor::Table, BookAuthor::BookId)
                        .to(Book::Table, Book::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(BookAuthor::Table, BookAuthor::AuthorId)
                        .to(Author::Table, Author::Id)
                        .on_delete(ForeignKeyAction::Restrict)
                )
                .to_owned()
        )
        .await?;

        manager.create_table(
            Table::create()
                .table(BookTag::Table)
                .col(integer(BookTag::BookId).not_null())
                .col(small_integer(BookTag::TagId).not_null())
                .primary_key(
                    Index::create()
                        .col(BookTag::BookId)
                        .col(BookTag::TagId)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(BookTag::Table, BookTag::BookId)
                        .to(Book::Table, Book::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(BookTag::Table, BookTag::TagId)
                        .to(Tag::Table, Tag::Id)
                        .on_delete(ForeignKeyAction::Restrict)
                )
                .to_owned()
        )
        .await?;

        manager.create_index(
            Index::create()
                .name("idx-index-book_id")
                .table(Chapter::Table)
                .col(Chapter::Index)
                .col(Chapter::BookId)
                .unique()
                .to_owned()
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Book::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Genre::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Series::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Chapter::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Author::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(BookGenre::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(BookAuthor::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(BookTag::Table).to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(Chapter::Table)
                    .name("idx-index-book_id")
                    .to_owned()
            )
            .await
    }
}