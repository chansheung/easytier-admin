//! `SeaORM` Entity, @generated

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "guest_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub token: String,
    pub password: String,
    pub created_by: String,
    pub max_use_count: i32,
    pub use_count: i32,
    pub is_active: bool,
    pub expires_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
