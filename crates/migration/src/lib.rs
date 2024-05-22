pub use sea_orm_migration::prelude::*;

mod m20220220_000001_migrate_resource_paths;
mod m20240522_000001_migrate_resource_paths;
mod path_migration;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Migration list.
        vec![
            Box::new(m20220220_000001_migrate_resource_paths::Migration),
            Box::new(m20240522_000001_migrate_resource_paths::Migration),
        ]
    }
    // Override the name of migration table.
    fn migration_table_name() -> sea_orm::DynIden {
        Alias::new("memospot_migrations").into_iden()
    }
}
