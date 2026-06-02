use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250602_000006_agent_nodes"
    }
}

#[derive(DeriveIden)]
pub enum AgentNodes {
    Table,
    Id,
    Name,
    VirtualIp,
    Description,
    LastSyncAt,
    LastSyncStatus,
    CreatedAt,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AgentNodes::Table)
                    .if_not_exists()
                    .col(pk_auto(AgentNodes::Id))
                    .col(string(AgentNodes::Name).not_null())
                    .col(string(AgentNodes::VirtualIp).not_null().unique_key())
                    .col(string(AgentNodes::Description).null())
                    .col(timestamp_with_time_zone(AgentNodes::LastSyncAt).null())
                    .col(string(AgentNodes::LastSyncStatus).not_null().default("unknown"))
                    .col(timestamp_with_time_zone(AgentNodes::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(AgentNodes::UpdatedAt).not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AgentNodes::Table).to_owned())
            .await
    }
}
