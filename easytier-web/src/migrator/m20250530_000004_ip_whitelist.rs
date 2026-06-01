use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250530_000004_ip_whitelist"
    }
}

#[derive(DeriveIden)]
pub enum IpWhitelist {
    Table,
    Id,
    Ip,
    Comment,
    CreatedBy,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(IpWhitelist::Table)
                    .if_not_exists()
                    .col(pk_auto(IpWhitelist::Id))
                    .col(string(IpWhitelist::Ip).not_null())
                    .col(string(IpWhitelist::Comment).null())
                    .col(string(IpWhitelist::CreatedBy).not_null())
                    .col(timestamp_with_time_zone(IpWhitelist::CreatedAt).not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(IpWhitelist::Table).to_owned())
            .await
    }
}
