//! User deletion route.

use rocket::{delete, http::Status};

use crate::database::{Database, Record};
use super::super::login::{Login, Owner, User};

#[utoipa::path(
    context_path = "/api/users",
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found"),
        (status = 401, description = "Invalid login session"),
        (status = 403, description = "Insufficient privileges"),
    ),
    security(("login" = [])),
    tag = "users",
)]

#[delete("/<id>")]
pub async fn route(user: Login<User>, db: &Database, id: &str) -> Status {
    // only allow deleting self if not owner
    let authorized = user.is(Owner) || user.id == id;
    if !authorized { return Status::Forbidden; }

    // query database to delete user
    let result: Option<Record> = db.delete(("user", id)).await
        .expect("error deleting user");

    // return success or not found status
    result.map(|_| Status::NoContent).unwrap_or(Status::NotFound)
}
