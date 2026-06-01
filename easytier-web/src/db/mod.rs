// sea-orm-cli generate entity -u sqlite:./et.db -o easytier-web/src/db/entity/ --with-serde both --with-copy-enums
#[allow(unused_imports)]
pub mod entity;

use entity::user_running_network_configs;
use sea_orm::{
    prelude::Expr, sea_query::OnConflict, ColumnTrait as _, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter as _, SqlxSqliteConnector, TransactionTrait as _,
};
use sea_orm_migration::MigratorTrait as _;
use sqlx::{migrate::MigrateDatabase as _, types::chrono, Sqlite, SqlitePool};

use crate::migrator;

pub type UserIdInDb = i32;

pub enum ListNetworkProps {
    All,
    EnabledOnly,
    DisabledOnly,
}

#[derive(Debug, Clone)]
pub struct Db {
    db_path: String,
    db: SqlitePool,
    orm_db: DatabaseConnection,
}

impl Db {
    pub async fn new<T: ToString>(db_path: T) -> anyhow::Result<Self> {
        let db = Self::prepare_db(db_path.to_string().as_str()).await?;
        let orm_db = SqlxSqliteConnector::from_sqlx_sqlite_pool(db.clone());
        migrator::Migrator::up(&orm_db, None).await?;

        Ok(Self {
            db_path: db_path.to_string(),
            db,
            orm_db,
        })
    }

    pub async fn memory_db() -> Self {
        Self::new(":memory:").await.unwrap()
    }

    #[tracing::instrument(ret)]
    async fn prepare_db(db_path: &str) -> anyhow::Result<SqlitePool> {
        if !Sqlite::database_exists(db_path).await.unwrap_or(false) {
            tracing::info!("Database not found, creating a new one");
            Sqlite::create_database(db_path).await?;
        }

        let db = sqlx::pool::PoolOptions::new()
            .max_lifetime(None)
            .idle_timeout(None)
            .connect(db_path)
            .await?;

        Ok(db)
    }

    pub fn inner(&self) -> SqlitePool {
        self.db.clone()
    }

    pub fn orm_db(&self) -> &DatabaseConnection {
        &self.orm_db
    }

    pub async fn insert_or_update_user_network_config<T: ToString>(
        &self,
        user_id: UserIdInDb,
        device_id: uuid::Uuid,
        network_inst_id: uuid::Uuid,
        network_config: T,
    ) -> Result<(), DbErr> {
        let txn = self.orm_db().begin().await?;

        use entity::user_running_network_configs as urnc;

        let on_conflict = OnConflict::column(urnc::Column::NetworkInstanceId)
            .update_columns([
                urnc::Column::NetworkConfig,
                urnc::Column::Disabled,
                urnc::Column::UpdateTime,
            ])
            .to_owned();
        let insert_m = urnc::ActiveModel {
            user_id: sea_orm::Set(user_id),
            device_id: sea_orm::Set(device_id.to_string()),
            network_instance_id: sea_orm::Set(network_inst_id.to_string()),
            network_config: sea_orm::Set(network_config.to_string()),
            disabled: sea_orm::Set(false),
            create_time: sea_orm::Set(chrono::Local::now().fixed_offset()),
            update_time: sea_orm::Set(chrono::Local::now().fixed_offset()),
            ..Default::default()
        };
        urnc::Entity::insert(insert_m)
            .on_conflict(on_conflict)
            .do_nothing()
            .exec(&txn)
            .await?;

        txn.commit().await
    }

    pub async fn delete_network_config(
        &self,
        user_id: UserIdInDb,
        network_inst_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        use entity::user_running_network_configs as urnc;

        urnc::Entity::delete_many()
            .filter(urnc::Column::UserId.eq(user_id))
            .filter(urnc::Column::NetworkInstanceId.eq(network_inst_id.to_string()))
            .exec(self.orm_db())
            .await?;

        Ok(())
    }

    pub async fn update_network_config_state(
        &self,
        user_id: UserIdInDb,
        network_inst_id: uuid::Uuid,
        disabled: bool,
    ) -> Result<entity::user_running_network_configs::Model, DbErr> {
        use entity::user_running_network_configs as urnc;

        urnc::Entity::update_many()
            .filter(urnc::Column::UserId.eq(user_id))
            .filter(urnc::Column::NetworkInstanceId.eq(network_inst_id.to_string()))
            .col_expr(urnc::Column::Disabled, Expr::value(disabled))
            .col_expr(
                urnc::Column::UpdateTime,
                Expr::value(chrono::Local::now().fixed_offset()),
            )
            .exec(self.orm_db())
            .await?;

        urnc::Entity::find()
            .filter(urnc::Column::UserId.eq(user_id))
            .filter(urnc::Column::NetworkInstanceId.eq(network_inst_id.to_string()))
            .one(self.orm_db())
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Network config not found for user {} and network instance {}",
                user_id, network_inst_id
            )))
    }

    pub async fn list_network_configs(
        &self,
        user_id: UserIdInDb,
        device_id: Option<uuid::Uuid>,
        props: ListNetworkProps,
    ) -> Result<Vec<user_running_network_configs::Model>, DbErr> {
        use entity::user_running_network_configs as urnc;

        let configs = urnc::Entity::find().filter(urnc::Column::UserId.eq(user_id));
        let configs = if matches!(
            props,
            ListNetworkProps::EnabledOnly | ListNetworkProps::DisabledOnly
        ) {
            configs
                .filter(urnc::Column::Disabled.eq(matches!(props, ListNetworkProps::DisabledOnly)))
        } else {
            configs
        };
        let configs = if let Some(device_id) = device_id {
            configs.filter(urnc::Column::DeviceId.eq(device_id.to_string()))
        } else {
            configs
        };

        let configs = configs.all(self.orm_db()).await?;

        Ok(configs)
    }

    pub async fn get_network_config(
        &self,
        user_id: UserIdInDb,
        device_id: &uuid::Uuid,
        network_inst_id: &String,
    ) -> Result<Option<user_running_network_configs::Model>, DbErr> {
        use entity::user_running_network_configs as urnc;

        let config = urnc::Entity::find()
            .filter(urnc::Column::UserId.eq(user_id))
            .filter(urnc::Column::DeviceId.eq(device_id.to_string()))
            .filter(urnc::Column::NetworkInstanceId.eq(network_inst_id))
            .one(self.orm_db())
            .await?;

        Ok(config)
    }

    pub async fn get_user_id<T: ToString>(
        &self,
        user_name: T,
    ) -> Result<Option<UserIdInDb>, DbErr> {
        use entity::users as u;

        let user = u::Entity::find()
            .filter(u::Column::Username.eq(user_name.to_string()))
            .one(self.orm_db())
            .await?;

        Ok(user.map(|u| u.id))
    }

    pub async fn get_user_id_by_token<T: ToString>(
        &self,
        token: T,
    ) -> Result<Option<UserIdInDb>, DbErr> {
        let token_str = token.to_string();

        if !token_str.starts_with("__guest_") {
            if let Some(user_id) = self.get_user_id(&token_str).await? {
                return Ok(Some(user_id));
            }
        }

        let now = chrono::Local::now().fixed_offset();
        let guest = entity::guest_tokens::Entity::find()
            .filter(entity::guest_tokens::Column::Token.eq(&token_str))
            .filter(entity::guest_tokens::Column::IsActive.eq(true))
            .filter(entity::guest_tokens::Column::ExpiresAt.gt(now))
            .one(self.orm_db())
            .await?;

        if let Some(g) = guest {
            if g.use_count >= g.max_use_count {
                tracing::warn!("Guest token {} exceeded max use count", token_str);
                return Ok(None);
            }

            let current_count = g.use_count;
            let mut active: entity::guest_tokens::ActiveModel = g.into();
            active.use_count = sea_orm::Set(current_count + 1);
            entity::guest_tokens::Entity::update(active)
                .exec(self.orm_db())
                .await?;

            let guest_username = format!("__guest_{}", &token_str[..8.min(token_str.len())]);
            let existing_user = entity::users::Entity::find()
                .filter(entity::users::Column::Username.eq(&guest_username))
                .one(self.orm_db())
                .await?;

            if let Some(u) = existing_user {
                Ok(Some(u.id))
            } else {
                let new_user = entity::users::ActiveModel {
                    username: sea_orm::Set(guest_username),
                    password: sea_orm::Set(password_auth::generate_hash(&token_str)),
                    ..Default::default()
                };
                let result = entity::users::Entity::insert(new_user)
                    .exec(self.orm_db())
                    .await?;
                Ok(Some(result.last_insert_id))
            }
        } else if let Some(s) = {
            entity::network_secrets::Entity::find()
                .filter(entity::network_secrets::Column::Secret.eq(&token_str))
                .filter(entity::network_secrets::Column::IsActive.eq(true))
                .one(self.orm_db())
                .await?
        } {
            if let Some(expires_at) = s.expires_at {
                if expires_at < chrono::Local::now().fixed_offset() {
                    return Ok(None);
                }
            }
            let secret_username = format!("__secret_{}", &token_str[..8.min(token_str.len())]);
            let existing_user = entity::users::Entity::find()
                .filter(entity::users::Column::Username.eq(&secret_username))
                .one(self.orm_db())
                .await?;

            if let Some(u) = existing_user {
                Ok(Some(u.id))
            } else {
                let new_user = entity::users::ActiveModel {
                    username: sea_orm::Set(secret_username),
                    password: sea_orm::Set(password_auth::generate_hash(&token_str)),
                    ..Default::default()
                };
                let result = entity::users::Entity::insert(new_user)
                    .exec(self.orm_db())
                    .await?;
                Ok(Some(result.last_insert_id))
            }
        } else {
            Ok(None)
        }
    }

}

#[cfg(test)]
mod tests {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter as _};

    use crate::db::{entity::user_running_network_configs, Db, ListNetworkProps};

    #[tokio::test]
    async fn test_user_network_config_management() {
        let db = Db::memory_db().await;
        let user_id = 1;
        let network_config = "test_config";
        let inst_id = uuid::Uuid::new_v4();
        let device_id = uuid::Uuid::new_v4();

        db.insert_or_update_user_network_config(user_id, device_id, inst_id, network_config)
            .await
            .unwrap();

        let result = user_running_network_configs::Entity::find()
            .filter(user_running_network_configs::Column::UserId.eq(user_id))
            .one(db.orm_db())
            .await
            .unwrap()
            .unwrap();
        println!("{:?}", result);
        assert_eq!(result.network_config, network_config);

        // overwrite the config
        let network_config = "test_config2";
        db.insert_or_update_user_network_config(user_id, device_id, inst_id, network_config)
            .await
            .unwrap();

        let result2 = user_running_network_configs::Entity::find()
            .filter(user_running_network_configs::Column::UserId.eq(user_id))
            .one(db.orm_db())
            .await
            .unwrap()
            .unwrap();
        println!("device: {}, {:?}", device_id, result2);
        assert_eq!(result2.network_config, network_config);

        assert_eq!(result.create_time, result2.create_time);
        assert_ne!(result.update_time, result2.update_time);

        assert_eq!(
            db.list_network_configs(user_id, Some(device_id), ListNetworkProps::All)
                .await
                .unwrap()
                .len(),
            1
        );

        db.delete_network_config(user_id, inst_id).await.unwrap();
        let result3 = user_running_network_configs::Entity::find()
            .filter(user_running_network_configs::Column::UserId.eq(user_id))
            .one(db.orm_db())
            .await
            .unwrap();
        assert!(result3.is_none());
    }
}
