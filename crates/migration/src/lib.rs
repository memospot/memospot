pub use sea_orm_migration::prelude::*;

mod m20220101_000001_migrate_resource_paths;
mod path;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Migration list.
        vec![Box::new(m20220101_000001_migrate_resource_paths::Migration)]
    }
    // Override the name of migration table.
    fn migration_table_name() -> sea_orm::DynIden {
        Alias::new("memospot_migrations").into_iden()
    }
}
