use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ip_whitelist")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub ip: String,
    pub comment: Option<String>,
    pub hostname: Option<String>,
    pub created_by: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
