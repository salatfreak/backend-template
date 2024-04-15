//! Configuration data structure and loading.

use rocket::{
    figment::{self, providers::{Env, Serialized}},
    http::uncased::{Uncased, UncasedStr},
};
use serde::Deserialize;

/// Load configuration, add defaults, add environment values, and parse it
pub fn load() -> Result<Config, figment::Error> {
    rocket::Config::figment()
        .join(Serialized::default("database.namespace", "default"))
        .join(Serialized::default("database.database", "default"))
        .join(Serialized::default("mail.pool_size", 1))
        .join(Serialized::default("files.path", "/usr/local/share/backend"))
        .join(Serialized::default("openapi.enable", false))
        .merge(Env::raw().map(convert_name).profile("global"))
        .extract()
}

/// Replace first underscore with a dot.
fn convert_name(name: &UncasedStr) -> Uncased<'_> {
    name.as_str().replacen("_", ".", 1).into()
}

/// Main config type.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub mail: MailConfig,
    pub files: FilesConfig,
    pub openapi: OpenAPIConfig,
}

/// Database config type.
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub address: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub namespace: String,
    pub database: String,
}

/// Mail config type.
#[derive(Debug, Deserialize)]
pub struct MailConfig {
    pub url: String,
    pub pool_size: u32,
    pub from: String,
}

/// Files config type.
#[derive(Debug, Deserialize)]
pub struct FilesConfig {
    pub path: String,
}

/// OpenAPI config type.
#[derive(Debug, Deserialize)]
pub struct OpenAPIConfig {
    pub enable: bool,
}
