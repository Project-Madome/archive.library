use sai::{Component, ComponentLifecycle, Injected};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::config::Config;

pub mod postgresql;

#[derive(Component)]
#[lifecycle]
pub struct DatabaseSet {
    #[injected]
    config: Injected<Config>,

    postgresql: Option<DatabaseConnection>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for DatabaseSet {
    async fn start(&mut self) {
        let postgresql = Self::connect_postgresql(self.config.postgres_url()).await;

        self.postgresql.replace(postgresql);
    }

    async fn stop(&mut self) {
        // log::info!("disconnect to database");
    }
}

impl DatabaseSet {
    async fn connect_postgresql(url: &str) -> DatabaseConnection {
        let option = ConnectOptions::new(url.to_string());

        Database::connect(option).await.expect("connect postgresql")
    }

    pub fn postgresql(&self) -> &DatabaseConnection {
        self.postgresql.as_ref().unwrap()
    }
}
