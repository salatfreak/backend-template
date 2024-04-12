//! User routes.

use crate::database::login::{Admin, Login, Owner, User};
use crate::database::{Database, Id};
use crate::mail::Mail;
use rocket::{get, http::Status, post, routes, serde::json::Json, Route};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

pub fn routes() -> Vec<Route> {
    routes![index, get, create]
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserOut {
    id: Id<String>,

    #[schema(example = "Alice")]
    name: String,
}

#[utoipa::path(
    context_path = "/api/users",
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
    context_path = "/api/users",
    responses(
        (status = 200, description = "User object", body = UserOut),
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

#[derive(Serialize, Deserialize, Validate, ToSchema)]
pub struct UserIn {
    #[schema(example = "Alice")]
    #[validate(length(min = 2))]
    name: String,

    #[schema(example = "alice@example.com")]
    #[validate(email)]
    email: String,

    #[schema(example = "supersecret")]
    #[validate(length(min = 8))]
    password: String,
}

#[utoipa::path(
    context_path = "/api/users",
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