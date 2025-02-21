pub use sea_orm_migration::prelude::*;

mod m20220220_000001_migrate_resource_paths;
mod m20240522_000002_migrate_resource_paths;
mod m20240525_000001_storage_settings;
mod m20250221_000001_memospot_v1;
mod resource_path;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Migration list.
        vec![
            Box::new(m20220220_000001_migrate_resource_paths::Migration),
            Box::new(m20240522_000002_migrate_resource_paths::Migration),
            Box::new(m20240525_000001_storage_settings::Migration),
            Box::new(m20250221_000001_memospot_v1::Migration),
        ]
    }
    // Override the name of migration table.
    fn migration_table_name() -> DynIden {
        Alias::new("memospot_migrations").into_iden()
    }
}
