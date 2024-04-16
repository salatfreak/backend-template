//! User routes.

use rocket::{get, http::Status, post, routes, serde::json::Json, Route};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::database::{Database, Id};
use crate::mail::Mail;
use super::{auth::register::RegisterIn, login::{Admin, Login, Owner, User}};

pub fn routes() -> Vec<Route> {
    routes![index, get, create]
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserOut {
    pub id: Id<String>,

    #[schema(example = "Alice")]
    pub name: String,
}

#[utoipa::path(
    context_path = "/api/users",
    responses(
        (status = 200, description = "List of users", body = Vec<UserOut>),
        (status = 401, description = "Invalid login session"),
        (status = 403, description = "Insufficient privileges"),
    ),
    security(("login" = [])),
    tag = "users",
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
    tag = "users",
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

#[utoipa::path(
    context_path = "/api/users",
    request_body = RegisterIn,
    responses(
        (status = 201, description = "User created"),
        (status = 401, description = "Invalid login session"),
        (status = 403, description = "Insufficient privileges"),
    ),
    security(("login" = [])),
    tag = "users",
)]

#[post("/", data = "<data>")]
async fn create(
    _user: Login<Owner>,
    db: &Database, mailer: &Mail, data: Json<RegisterIn>,
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
