//! Migrate resource internal paths from absolute to relative.
//!
//! This may be needed for Memos upgrades that skipped version 0.18.2 and 0.19.0.
//! The path migrator is not present in Memos 0.19.1 and later, so we need a custom
//! implementation to improve data portability.
//!
//! Notes:
//! - This migration does data manipulation.
//! - Migrating 300k resources takes about 20 seconds on a modern NVMe SSD and a decent CPU.

use itertools::Itertools;
use log::{debug, info, LevelFilter};
use sea_orm::entity::prelude::*;
use sea_orm::query::*;
use sea_orm_migration::prelude::*;
use std::env::consts::OS;

use crate::path;

use entity::resource;
use entity::resource::Entity as Resource;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Build a list of known paths to check for absolute resource paths.
fn build_path_list() -> Vec<String> {
    let data_path = path::get_app_data_path("memospot");
    let bin = std::env::current_exe().unwrap();
    let cwd = bin.parent().unwrap().to_path_buf();

    let mut paths: Vec<String> = vec![
        "/var/opt/memos/".to_string(),
        path::norm_suffix(data_path.to_string_lossy().as_ref()),
        path::norm_suffix(cwd.to_string_lossy().as_ref()),
    ];
    if OS == "windows" {
        if let Ok(program_data) = std::env::var("PROGRAMDATA") {
            paths.push(format!("{}\\memos\\", program_data));
        }
    }
    paths.into_iter().unique().collect()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        info!("::Database Migrator:: Migrating resource internal paths from absolute to relative.");
        let db = manager.get_connection();
        let paths: Vec<String> = build_path_list();

        // Find all resources with internal paths that are not null and have no blob.
        let resources = Resource::find()
            .columns([resource::Column::Id, resource::Column::InternalPath])
            .filter(
                Condition::all()
                    .add(resource::Column::Blob.is_null())
                    .add(resource::Column::InternalPath.is_not_null())
                    .add(resource::Column::InternalPath.ne("")),
            )
            .all(db)
            .await?;

        let mut migrated_count = 0;
        let total_resources = resources.len();
        let log_step = match total_resources {
            0..=100 => 1,
            101..=1000 => 10,
            1001..=10000 => 20,
            10001..=100000 => 50,
            100001..=1000000 => 100,
            _ => 100000,
        };
        let log_interval = total_resources / log_step;

        let transaction = db.begin().await?;
        for resource in resources {
            let mut new_path = resource.internal_path.clone();

            // Strip known path prefixes.
            for p in &paths {
                new_path = new_path.trim_start_matches(p).to_string();
            }

            new_path = path::to_slash(&new_path);

            // Fall back: strip everything before "/assets/".
            if new_path.contains("/assets/") {
                if let Some(file_name) = new_path.split("/assets/").collect::<Vec<&str>>().pop()
                {
                    new_path = "assets/".to_string() + file_name;
                }
            }

            new_path = new_path.trim_start_matches('/').to_string();

            // Update only if the path has changed.
            if new_path != resource.internal_path {
                Resource::update_many()
                    .col_expr(resource::Column::InternalPath, Expr::value(&new_path))
                    .filter(resource::Column::Id.eq(resource.id))
                    .exec(&transaction)
                    .await?;
            }

            migrated_count += 1;
            if migrated_count < 50 || migrated_count % log_interval == 0 {
                match log::max_level() {
                    LevelFilter::Info => {
                        info!(
                            "[Running] Migrated {}/{} paths.",
                            migrated_count, total_resources,
                        );
                    }
                    LevelFilter::Debug => {
                        debug!(
                            "[Running] Migrated {}/{} paths.\nLast: {} => {}",
                            migrated_count, total_resources, &resource.internal_path, new_path
                        );
                    }
                    _ => {}
                }
            }
        }
        transaction.commit().await?;
        Ok(())
    }

    /// This migration is not reversible.
    ///
    /// Also, even older Memos's versions are able to load assets with relative paths,
    /// they just don't store them that way.
    ///
    /// Returns `Ok` without doing anything.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
