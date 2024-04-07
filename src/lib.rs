//! Backend Template
//!
//! This package contains a template for building application backends using
//! the Rocket framework.

#![warn(rust_2018_idioms, missing_docs)]

use rocket::{get, routes, serde::json::{json, Value}, Build, Rocket};

/// Build the rocket instance ready to be ignited and launched.
pub fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![index])
}

/// Serve some dummy JSON on the index route.
#[get("/")]
fn index() -> Value {
    json![{"message": "Hello, world!"}]
}
