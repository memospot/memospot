//! Migrate to Memospot v1.0.0.
//!
//! This migration just records the migration event to know that we're at version 1.0.0.

use sea_orm::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(()) // Not reversible.
    }
}
