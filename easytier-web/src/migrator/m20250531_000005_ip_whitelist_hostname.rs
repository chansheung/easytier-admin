use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250531_000005_ip_whitelist_hostname"
    }
}

#[derive(DeriveIden)]
enum IpWhitelist {
    Table,
    Hostname,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(IpWhitelist::Table)
                    .add_column_if_not_exists(string_null(IpWhitelist::Hostname))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(IpWhitelist::Table)
                    .drop_column(IpWhitelist::Hostname)
                    .to_owned(),
            )
            .await
    }
}
