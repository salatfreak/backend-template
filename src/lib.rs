//! Backend Template
//!
//! This package contains a template for building application backends using
//! the Rocket framework.

#![warn(rust_2018_idioms, missing_docs)]

use rocket::{Build, Rocket};

mod files;

/// Build the rocket instance ready to be ignited and launched.
pub fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(files::mount("static/http"))
}
