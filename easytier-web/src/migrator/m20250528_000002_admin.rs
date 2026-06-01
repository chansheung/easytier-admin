use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250528_000002_admin"
    }
}

#[derive(DeriveIden)]
pub enum BlockedDevices {
    Table,
    Id,
    DeviceId,
    MachineId,
    UserId,
    BlockedBy,
    Reason,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum GuestTokens {
    Table,
    Id,
    Token,
    Password,
    CreatedBy,
    MaxUseCount,
    UseCount,
    IsActive,
    ExpiresAt,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(BlockedDevices::Table)
                    .col(pk_auto(BlockedDevices::Id).not_null())
                    .col(string(BlockedDevices::DeviceId).not_null())
                    .col(string(BlockedDevices::MachineId).not_null())
                    .col(integer(BlockedDevices::UserId).not_null())
                    .col(string(BlockedDevices::BlockedBy).not_null())
                    .col(string(BlockedDevices::Reason).not_null())
                    .col(timestamp_with_time_zone(BlockedDevices::CreatedAt).not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_blocked_devices_machine_id")
                    .table(BlockedDevices::Table)
                    .col(BlockedDevices::MachineId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(GuestTokens::Table)
                    .col(pk_auto(GuestTokens::Id).not_null())
                    .col(string(GuestTokens::Token).not_null())
                    .col(string(GuestTokens::Password).not_null())
                    .col(string(GuestTokens::CreatedBy).not_null())
                    .col(integer(GuestTokens::MaxUseCount).not_null().default(10))
                    .col(integer(GuestTokens::UseCount).not_null().default(0))
                    .col(boolean(GuestTokens::IsActive).not_null().default(true))
                    .col(timestamp_with_time_zone(GuestTokens::ExpiresAt).not_null())
                    .col(timestamp_with_time_zone(GuestTokens::CreatedAt).not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_guest_tokens_token")
                    .table(GuestTokens::Table)
                    .col(GuestTokens::Token)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BlockedDevices::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(GuestTokens::Table).to_owned())
            .await?;
        Ok(())
    }
}
