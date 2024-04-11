//! Hierarchy of API routes.

use crate::database::login::{Admin, Login, Owner, User};
use crate::database::{Database, Id};
use crate::mail::Mail;
use rocket::{
    fairing::AdHoc, get, http::Status, post, routes, serde::json::Json
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn mount() -> AdHoc {
    AdHoc::on_ignite("API Routes", |rocket| async {
        rocket.mount("/api", routes![index, get, create])
    })
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserOut {
    id: Id<String>,
    #[schema(example = "Alice")]
    name: String,
}

#[utoipa::path(
    context_path = "/api",
    responses(
        (status = 200, description = "List of users", body = Vec<UserOut>),
        (status = 401, description = "Invalid login session"),
        (status = 403, description = "Insufficient privileges"),
    ),
    security(("login" = [])),
)]

#[get("/")]
async fn index(_user: Login<Admin>, db: &Database) -> Json<Vec<UserOut>> {
    let users: Vec<UserOut> = db.select("user").await
        .expect("error retrieving users");
    Json(users)
}

#[utoipa::path(
    context_path = "/api",
    responses(
        (status = 200, description = "User object", body = User),
        (status = 404, description = "User not found"),
        (status = 401, description = "Invalid login session"),
        (status = 403, description = "Insufficient privileges"),
    ),
    security(("login" = [])),
)]

#[get("/<id>")]
async fn get(
    user: Login<User>, db: &Database, id: &str
) -> Result<Json<UserOut>, Status> {
    // only allow reading own info if not admin
    let authorized = user.is(Admin) || user.id == id;
    if !authorized { return Err(Status::Forbidden); }

    // fetch and return user
    match db.select(("user", id)).await.expect("error retrieving user") {
        Some(u) => Ok(Json(u)),
        None => Err(Status::NotFound),
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserIn {
    #[schema(example = "Alice")]
    name: String,
}

#[utoipa::path(
    context_path = "/api",
    request_body = UserIn,
    responses(
        (status = 201, description = "User created"),
        (status = 401, description = "Invalid login session"),
        (status = 403, description = "Insufficient privileges"),
    ),
    security(("login" = [])),
)]

#[post("/", data = "<data>")]
async fn create(
    _user: Login<Owner>,
    db: &Database, mailer: &Mail, data: Json<UserIn>,
) -> Status {
    // create entry
    db.create::<Vec<UserOut>>("user").content(data.into_inner()).await
        .expect("error creating user");

    // send email
    let email = mailer
        .template("verify-account", &[("token", "<supersecret>")])
        .await.expect("error loading email template");
    mailer.send("test@example.com", email)
        .await.expect("error sending email");

    // return created status
    Status::Created
}
