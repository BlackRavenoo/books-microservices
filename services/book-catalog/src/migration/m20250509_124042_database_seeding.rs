use sea_orm_migration::prelude::*;
use std::fs::File;
use std::io::Read;
use csv::Reader;

use super::m20220101_000001_create_table::{Genre, Tag};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut file = File::open("migration-data/genres.csv")
            .map_err(|e| DbErr::Custom(format!("Failed to open genres.csv: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| DbErr::Custom(format!("Failed to read genres.csv: {}", e)))?;
        
        let mut rdr = Reader::from_reader(contents.as_bytes());
        for result in rdr.records() {
            let record = result.map_err(|e| DbErr::Custom(format!("Failed to parse CSV record: {}", e)))?;
            if let Some(name) = record.get(0) {
                if !name.is_empty() {
                    let insert = Query::insert()
                        .into_table(Genre::Table)
                        .columns([Genre::Name])
                        .values_panic([name.into()])
                        .to_owned();

                    manager.exec_stmt(insert).await?;
                }
            }
        }
        
        let mut file = File::open("migration-data/tags.csv")
            .map_err(|e| DbErr::Custom(format!("Failed to open tags.csv: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| DbErr::Custom(format!("Failed to read tags.csv: {}", e)))?;
        
        let mut rdr = Reader::from_reader(contents.as_bytes());
        for result in rdr.records() {
            let record = result.map_err(|e| DbErr::Custom(format!("Failed to parse CSV record: {}", e)))?;
            if let Some(name) = record.get(0) {
                if !name.is_empty() {
                    let insert = Query::insert()
                        .into_table(Tag::Table)
                        .columns([Tag::Name])
                        .values_panic([name.into()])
                        .to_owned();

                    manager.exec_stmt(insert).await?;
                }
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.truncate_table(
            Table::truncate()
                .table(Genre::Table)
                .to_owned()
        ).await?;

        manager.truncate_table(
            Table::truncate()
                .table(Tag::Table)
                .to_owned()
        ).await
    }
}