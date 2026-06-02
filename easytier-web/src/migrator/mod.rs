use sea_orm_migration::prelude::*;

mod m20241029_000001_init;
mod m20250528_000002_admin;
mod m20250530_000003_network_secrets;
mod m20250530_000004_ip_whitelist;
mod m20250531_000005_ip_whitelist_hostname;
mod m20250602_000006_agent_nodes;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241029_000001_init::Migration),
            Box::new(m20250528_000002_admin::Migration),
            Box::new(m20250530_000003_network_secrets::Migration),
            Box::new(m20250530_000004_ip_whitelist::Migration),
            Box::new(m20250531_000005_ip_whitelist_hostname::Migration),
            Box::new(m20250602_000006_agent_nodes::Migration),
        ]
    }
}
