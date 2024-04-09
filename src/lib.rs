//! Backend Template
//!
//! This package contains a template for building application backends using
//! the Rocket framework.

#![warn(rust_2018_idioms, missing_docs)]

use rocket::{Build, Rocket};

mod database;
mod mail;
mod api;
mod doc;
mod files;

/// Build the rocket instance ready to be ignited and launched.
pub fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(database::mount(
            "localhost:8080", "root", "root", "backend", "backend"
        ))
        .attach(mail::mount(
            "smtp://localhost:2525", 1, "sender@example.com",
            "static/mail".into(),
        ))
        .attach(api::mount())
        .attach(doc::mount())
        .attach(files::mount("static/http"))
}
