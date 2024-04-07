//! Hierarchy of API routes.

use rocket::{fairing::AdHoc, get, routes, serde::json::{json, Value}};

pub fn mount() -> AdHoc {
    AdHoc::on_ignite("API Routes", |rocket| async {
        rocket.mount("/api", routes![index])
    })
}

#[get("/")]
fn index() -> Value {
    json![{"message": "Hello, world!"}]
}