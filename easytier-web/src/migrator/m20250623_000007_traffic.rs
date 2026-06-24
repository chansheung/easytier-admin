use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250623_000007_traffic"
    }
}

#[derive(DeriveIden)]
enum TrafficQuota {
    Table,
    Id,
    Ip,
    PeriodType,
    LimitBytes,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TrafficUsage {
    Table,
    Id,
    Ip,
    PeriodType,
    PeriodKey,
    Bytes,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TrafficQuota::Table)
                    .if_not_exists()
                    .col(pk_auto(TrafficQuota::Id))
                    .col(string(TrafficQuota::Ip).not_null())
                    .col(string(TrafficQuota::PeriodType).not_null())
                    .col(big_integer(TrafficQuota::LimitBytes).not_null())
                    .col(boolean(TrafficQuota::Enabled).not_null().default(true))
                    .col(timestamp_with_time_zone(TrafficQuota::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(TrafficQuota::UpdatedAt).not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(TrafficUsage::Table)
                    .if_not_exists()
                    .col(pk_auto(TrafficUsage::Id))
                    .col(string(TrafficUsage::Ip).not_null())
                    .col(string(TrafficUsage::PeriodType).not_null())
                    .col(string(TrafficUsage::PeriodKey).not_null())
                    .col(big_integer(TrafficUsage::Bytes).not_null().default(0))
                    .col(timestamp_with_time_zone(TrafficUsage::UpdatedAt).not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TrafficQuota::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TrafficUsage::Table).to_owned())
            .await
    }
}
