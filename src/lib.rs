//! Backend Template
//!
//! This package contains a template for building application backends using
//! the Rocket framework.

#![warn(rust_2018_idioms, missing_docs)]

use std::path::PathBuf;
use rocket::{figment, Build, Rocket};

mod config;
mod database;
mod mail;
mod api;
mod doc;
mod files;

/// Build the rocket instance ready to be ignited and launched.
pub fn rocket() -> Rocket<Build> {
    // load config and panic on errors
    let config = match config::load() {
        Ok(config) => config,
        Err(figment::Error { path, kind, .. }) =>
            panic!("Configuration error: {} in {}", kind, path.join(".")),
    };
    let files_path = PathBuf::from(config.files.path);

    // build rocket instance
    let mut rocket = rocket::build()
        .attach(database::mount(config.database))
        .attach(mail::mount(config.mail, files_path.join("mail")))
        .attach(api::mount(config.api))
        .attach(files::mount(files_path.join("http")));

    // mount OpenAPI spec and UI if requested
    if config.openapi.enable {
        rocket = rocket.attach(doc::mount())
    }

    rocket
}
