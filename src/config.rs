use std::{env, fmt::Debug, str::FromStr};

use sai::{Component, ComponentLifecycle};

fn env<T>(key: &str) -> T
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let var = env::var(key).expect("Please set dotenv");

    var.parse().expect("Please set dotenv to valid value")
}

#[derive(Debug, Component)]
#[lifecycle]
pub struct Config {
    port: Option<u16>,

    postgres_url: Option<String>,
    /* postgres_port: Option<String>,
    postgres_host: Option<String>,
    postgres_user: Option<String>,
    postgres_pw: Option<String>,
    postgres_db: Option<String>, */
    madome_auth_url: Option<String>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for Config {
    async fn start(&mut self) {
        dotenv::dotenv().ok();

        self.port.replace(env("PORT"));

        let pg_port: u16 = env("POSTGRES_PORT");
        let pg_host: String = env("POSTGRES_HOST");
        let pg_user: String = env("POSTGRES_USER");
        let pg_pw: String = env("POSTGRES_PW");
        let pg_db: String = env("POSTGRES_DB");
        let pg_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            pg_user, pg_pw, pg_host, pg_port, pg_db
        );
        self.postgres_url.replace(pg_url);

        self.madome_auth_url.replace(env("MADOME_AUTH_URL"));

        log::info!("{:?}", self);
    }
}

impl Config {
    pub fn port(&self) -> u16 {
        self.port.unwrap()
    }

    pub fn postgres_url(&self) -> &str {
        self.postgres_url.as_ref().unwrap()
    }

    pub fn auth_url(&self) -> &str {
        self.madome_auth_url.as_ref().unwrap()
    }
}
