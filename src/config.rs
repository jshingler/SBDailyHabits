
use serde::Deserialize;
// use crate::config::Config;
use serde_java_properties::from_reader;
use std::error::Error;
use std::fs::File;
use once_cell::sync::Lazy;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "app.name")]
    pub name: String,

    #[serde(rename = "app.version")]
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    #[serde(rename = "database.user")]
    pub user: String,

    #[serde(rename = "database.password")]
    pub password: String,

    #[serde(rename = "database.host")]
    pub host: String,

    #[serde(rename = "database.port")]
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct NotionApiConfig {
    #[serde(rename = "notion.api.url")]
    pub url: String,

    #[serde(rename = "notion.api.version")]
    pub version: String,

    #[serde(rename = "notion.api.token")]
    pub token: String,

    #[serde(rename = "daily.database.id")]
    pub daily_database_id: String,

    #[serde(rename = "habits.database.id")]
    pub habits_database_id: String,

    #[serde(rename = "habits.master.database.id")]
    pub habits_master_database_id: String,

    #[serde(rename = "daily.stats.page.id")]
    pub daily_stats_page_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub app: AppConfig,

    #[serde(flatten)]
    pub database: DatabaseConfig,

    #[serde(flatten)]
    pub notion: NotionApiConfig,
}

fn load_config() -> Result<Config, Box<dyn Error>> {
    let file = File::open("config/config.properties")?;
    let config: Config = from_reader(file)?;
    Ok(config)
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    load_config().expect("Failed to load configuration")
});