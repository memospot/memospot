//! Migrate resource internal paths from absolute to relative.
//!
//! This may be needed for Memos upgrades that skipped version 0.18.2 and 0.19.0.
//! The path migrator is not present in Memos 0.19.1 and later, so we need a custom
//! implementation to improve data portability.
//!
//! Notes:
//! - This migration does data manipulation.
//! - Migrating 300k resources takes about 20 seconds on a modern NVMe SSD and a decent CPU.
//! - Valid from Memos v0.22.0 onwards.

use log::{debug, info, LevelFilter};
use sea_orm::*;
use sea_orm_migration::prelude::*;

use crate::resource_path::{self};
mod resource {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "resource")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub creator_id: i32,
        pub created_ts: i64,
        pub updated_ts: i64,
        pub filename: String,
        #[sea_orm(column_type = "VarBinary(StringLen::None)", nullable)]
        pub blob: Option<Vec<u8>>,
        pub r#type: String,
        pub size: i32,
        pub memo_id: Option<i32>,
        pub uid: String,
        pub storage_type: String,
        pub reference: String,
        pub payload: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
use resource::Entity as Resource;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        info!("::Database Migrator:: Migrating resource internal paths from absolute to relative.  [>= v0.22.0]");

        // Check the `resource` table schema.
        {
            if !manager.has_table(Resource.table_name()).await? {
                return Ok(()); // Schema not supported.
            }

            for column in [
                resource::Column::Id,
                resource::Column::Blob,
                resource::Column::Reference,
            ] {
                if !manager
                    .has_column(Resource.table_name(), column.as_str())
                    .await?
                {
                    return Ok(()); // Schema not supported.
                }
            }
        }

        let db = manager.get_connection();

        // Find eligible resources.
        let resources = Resource::find()
            .columns([resource::Column::Id, resource::Column::Reference])
            .filter(
                Condition::all()
                    .add(resource::Column::Blob.is_null())
                    .add(resource::Column::Reference.is_not_null())
                    .add(resource::Column::Reference.ne(""))
                    .not()
                    .add(resource::Column::Reference.starts_with("assets/"))
                    .not()
                    .add(resource::Column::Reference.starts_with("http")),
            )
            .all(db)
            .await?;

        let total_resources: usize = resources.len();
        let log_step = match total_resources {
            0..=100 => 1,
            101..=1000 => 10,
            1001..=10000 => 20,
            10001..=100000 => 50,
            100001..=1000000 => 100,
            _ => 100000,
        };
        let log_interval = total_resources / log_step;

        let paths: Vec<String> = resource_path::build_path_list();

        let mut migrated_count = 0;
        let transaction = db.begin().await?;
        for resource in resources {
            let mut new_path = resource.reference.clone();

            // Strip known path prefixes.
            for p in &paths {
                new_path = new_path.trim_start_matches(p).to_string();
            }

            new_path = resource_path::to_slash(&new_path);

            // Fall back: strip everything before "/assets/".
            if new_path.contains("/assets/") {
                if let Some(file_name) = new_path.split("/assets/").collect::<Vec<&str>>().pop()
                {
                    new_path = "assets/".to_string() + file_name;
                }
            }

            new_path = new_path.trim_start_matches('/').to_string();

            // Update only if the path has changed.
            if new_path != resource.reference {
                Resource::update_many()
                    .col_expr(resource::Column::Reference, Expr::value(&new_path))
                    .filter(resource::Column::Id.eq(resource.id))
                    .exec(&transaction)
                    .await?;
            }

            migrated_count += 1;
            if migrated_count < 50 || migrated_count % log_interval == 0 {
                match log::max_level() {
                    LevelFilter::Info => {
                        info!("[Running] Migrated {migrated_count}/{total_resources} paths.");
                    }
                    LevelFilter::Debug => {
                        debug!(
                            "[Running] Migrated {migrated_count}/{total_resources} paths.\nLast: {} => {}",
                            &resource.reference, new_path
                        );
                    }
                    _ => {}
                }
            }
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(()) // Not reversible.
    }
}
