//! User retrieval route.

use rocket::{get, http::Status, serde::json::Json};

use crate::database::Database;
use super::{super::login::{Admin, Login, User}, components::UserOut};

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
pub async fn route(
    user: Login<User>, db: &Database, id: &str
) -> Result<Json<UserOut>, Status> {
    // only allow reading own info if not admin
    let authorized = user.is(Admin) || user.id == id;
    if !authorized { return Err(Status::Forbidden); }

    // fetch user from database
    let user: Option<UserOut> = db.select(("user", id)).await
        .expect("error retrieving user");

    // return user or not found status
    user.map(Json).ok_or(Status::NotFound)
}
