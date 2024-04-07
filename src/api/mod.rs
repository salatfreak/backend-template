//! Hierarchy of API routes.

use crate::database::{Database, Id};
use rocket::{fairing::AdHoc, get, http::Status, post, routes, serde::json::Json};
use serde::{Deserialize, Serialize};

pub fn mount() -> AdHoc {
    AdHoc::on_ignite("API Routes", |rocket| async {
        rocket.mount("/api", routes![index, get, create])
    })
}

#[derive(Serialize, Deserialize)]
struct User {
    id: Id<String>,
    name: String,
    age: usize,
}

#[get("/")]
async fn index(db: &Database) -> Json<Vec<User>> {
    let users: Vec<User> = db.select("user").await
        .expect("error retrieving users");
    Json(users)
}

#[get("/<id>")]
async fn get(db: &Database, id: &str) -> Result<Json<User>, Status> {
    match db.select(("user", id)).await.expect("error retrieving user") {
        Some(user) => Ok(Json(user)),
        None => Err(Status::NotFound),
    }
}

#[derive(Serialize, Deserialize)]
struct NewUser {
    name: String,
    age: usize,
}

#[post("/", data = "<data>")]
async fn create(
    db: &Database, data: Json<NewUser>,
) -> Status {
    db.create::<Vec<User>>("user").content(data.into_inner()).await
        .expect("error creating user");
    Status::Created
}
