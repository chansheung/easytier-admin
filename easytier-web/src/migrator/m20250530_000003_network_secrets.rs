use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250530_000003_network_secrets"
    }
}

#[derive(DeriveIden)]
enum NetworkSecrets {
    Table,
    Id,
    Name,
    Secret,
    CreatedBy,
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
                    .table(NetworkSecrets::Table)
                    .if_not_exists()
                    .col(pk_auto(NetworkSecrets::Id))
                    .col(string(NetworkSecrets::Name).not_null())
                    .col(string(NetworkSecrets::Secret).not_null())
                    .col(string(NetworkSecrets::CreatedBy).not_null())
                    .col(boolean(NetworkSecrets::IsActive).not_null().default(true))
                    .col(timestamp_with_time_zone(NetworkSecrets::ExpiresAt).null())
                    .col(timestamp_with_time_zone(NetworkSecrets::CreatedAt).not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NetworkSecrets::Table).to_owned())
            .await
    }
}
