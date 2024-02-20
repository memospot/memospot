//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "webhook")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub created_ts: i64,
    pub updated_ts: i64,
    pub row_status: String,
    pub creator_id: i32,
    pub name: String,
    pub url: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
