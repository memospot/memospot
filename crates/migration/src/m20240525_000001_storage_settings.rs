//! Migrate storage settings from Memos <=v0.21.1 to Memos >=v0.22.0.
//!
//! Notes:
//! - This migration does data manipulation.

use log::{debug, info};
use sea_orm::*;
use sea_orm_migration::prelude::*;

mod system_setting {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "system_setting")]
    pub struct Model {
        #[sea_orm(unique, primary_key)]
        pub name: String,
        pub value: String,
        pub description: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
use system_setting::Entity as SystemSetting;

mod storage {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "storage")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub r#type: String,
        pub config: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
use storage::Entity as Storage;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        info!("::Database Migrator:: Migrating storage settings.  [>= v0.22.0]");

        // Check the `system_setting` table schema.
        {
            if !manager.has_table(SystemSetting.table_name()).await? {
                return Ok(()); // Schema not supported.
            }

            for column in [
                system_setting::Column::Name,
                system_setting::Column::Value,
                system_setting::Column::Description,
            ] {
                if !manager
                    .has_column(SystemSetting.table_name(), column.as_str())
                    .await?
                {
                    return Ok(()); // Schema not supported.
                }
            }
        }

        let db = manager.get_connection();

        if let Some(current_storage) = SystemSetting::find()
            .filter(system_setting::Column::Name.eq("STORAGE"))
            .one(db)
            .await?
        {
            if current_storage.value.contains("storageType")
                && current_storage.value.contains("filepathTemplate")
                && current_storage.value.contains("uploadSizeLimitMb")
            {
                // Storage values seems valid. Mark the migration as completed.
                debug!("Found v0.22 storage settings. Marking migration as completed.");
                return Ok(());
            }
        }

        let storage_service_id = match SystemSetting::find()
            .filter(system_setting::Column::Name.eq("storage-service-id"))
            .one(db)
            .await?
        {
            Some(s) => s.value,
            None => "-1".to_string(),
        };
        let storage_type = match storage_service_id.parse::<i32>() {
            Err(_) => "LOCAL",
            Ok(-1) => "LOCAL",
            Ok(0) => "DATABASE",
            Ok(_) => "S3",
        };

        let local_storage_path = match SystemSetting::find()
            .filter(system_setting::Column::Name.eq("local-storage-path"))
            .one(db)
            .await?
        {
            Some(s) => s.value.replace('"', ""), // Remove JSON quotes.
            None => r#"assets/{year}{month}/{timestamp}_{filename}"#.to_string(),
        };

        let max_upload_size = match SystemSetting::find()
            .filter(system_setting::Column::Name.eq("max-upload-size-mib"))
            .one(db)
            .await?
        {
            Some(s) => s.value,
            None => "32".to_string(),
        };

        debug!(
            "storage-service-id: {} ({})",
            storage_service_id, storage_type
        );
        debug!("local-storage-path: {}", local_storage_path);
        debug!("max-upload-size-mib: {}", max_upload_size);

        let transaction = db.begin().await?;

        // Create a setting in the new format.
        let storage_setting = system_setting::ActiveModel {
            name: Set("STORAGE".to_string()),
            value: Set(serde_json::json!({
                "storageType": storage_type,
                "filepathTemplate": local_storage_path,
                "uploadSizeLimitMb": max_upload_size,
            })
            .to_string()),
            description: Set("".to_string()),
        };
        SystemSetting::insert(storage_setting)
            .exec(&transaction)
            .await?;

        // Remove old settings.
        SystemSetting::delete_many()
            .filter(
                Condition::any()
                    .add(system_setting::Column::Name.eq("local-storage-path"))
                    .add(system_setting::Column::Name.eq("max-upload-size-mib"))
                    .add(system_setting::Column::Name.eq("storage-service-id")),
            )
            .exec(&transaction)
            .await?;

        transaction.commit().await?;

        // S3 config migration //
        debug!("Attempting to migrate S3 config.");

        // Check the `storage` table schema.
        {
            if !manager.has_table(Storage.table_name()).await? {
                return Ok(()); // Schema not supported.
            }

            for col in [
                storage::Column::Id,
                storage::Column::Name,
                storage::Column::Type,
                storage::Column::Config,
            ] {
                if !manager
                    .has_column(Storage.table_name(), col.as_str())
                    .await?
                {
                    return Ok(()); // Schema not supported.
                }
            }
        }

        let active_id = storage_service_id.parse::<i32>().unwrap_or(-1);

        // Get active config matching the storage service id. If not found, get first config.
        let active_config = match Storage::find()
            .filter(storage::Column::Id.eq(active_id))
            .one(db)
            .await?
        {
            Some(s) => s.config,
            None => {
                match Storage::find()
                    .filter(storage::Column::Type.eq("S3"))
                    .one(db)
                    .await?
                {
                    Some(s) => s.config,
                    None => "{}".to_string(),
                }
            }
        };

        if active_config == "{}" {
            return Ok(()); // No S3 config found.
        }

        let parsed_config = serde_json::from_str(&active_config);
        if parsed_config.is_err() {
            return Ok(()); // Unable to parse S3 config.
        }

        let storage_config: serde_json::Value = parsed_config.unwrap_or_default();
        let endpoint = storage_config
            .get("endPoint")
            .unwrap_or(&serde_json::Value::Null)
            .as_str()
            .unwrap_or_default();
        let region = storage_config
            .get("region")
            .unwrap_or(&serde_json::Value::Null)
            .as_str()
            .unwrap_or_default();
        let access_key = storage_config
            .get("accessKey")
            .unwrap_or(&serde_json::Value::Null)
            .as_str()
            .unwrap_or_default();
        let secret_key = storage_config
            .get("secretKey")
            .unwrap_or(&serde_json::Value::Null)
            .as_str()
            .unwrap_or_default();
        let bucket = storage_config
            .get("bucket")
            .unwrap_or(&serde_json::Value::Null)
            .as_str()
            .unwrap_or_default();

        let s3_config = serde_json::json!({
            "accessKeyId": access_key,
            "accessKeySecret": secret_key,
            "endpoint": endpoint,
            "region": region,
            "bucket": bucket,
        });

        let transaction = db.begin().await?;

        // Update the setting.
        let storage_setting = system_setting::ActiveModel {
            name: Set("STORAGE".to_string()),
            value: Set(serde_json::json!({
                "storageType": storage_type,
                "filepathTemplate": local_storage_path,
                "uploadSizeLimitMb": max_upload_size,
                "s3Config": s3_config,
            })
            .to_string()),
            description: Set("".to_string()),
        };
        SystemSetting::update(storage_setting)
            .exec(&transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(()) // Not reversible.
    }
}
