//! `SeaORM` Entity, @generated

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "blocked_devices")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub device_id: String,
    pub machine_id: String,
    pub user_id: i32,
    pub blocked_by: String,
    pub reason: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
