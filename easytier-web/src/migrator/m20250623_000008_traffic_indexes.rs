use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250623_000008_traffic_indexes"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_traffic_quota_ip_period")
                    .table(TrafficQuota::Table)
                    .col(TrafficQuota::Ip)
                    .col(TrafficQuota::PeriodType)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_traffic_usage_ip_period_key")
                    .table(TrafficUsage::Table)
                    .col(TrafficUsage::Ip)
                    .col(TrafficUsage::PeriodType)
                    .col(TrafficUsage::PeriodKey)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_traffic_usage_ip_period_key")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_traffic_quota_ip_period")
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum TrafficQuota {
    Table,
    Ip,
    PeriodType,
}

#[derive(DeriveIden)]
enum TrafficUsage {
    Table,
    Ip,
    PeriodType,
    PeriodKey,
}
