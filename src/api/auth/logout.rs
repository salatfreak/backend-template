//! Logout route.

use rocket::{http::Status, post};

use crate::database::Database;
use super::super::login::{Login, User};

#[utoipa::path(
    context_path = "/api/auth",
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Invalid login session"),
    ),
    security(("login" = [])),
    tag = "authentication",
)]

#[post("/logout")]
pub async fn route(db: &Database, user: Login<User>) -> Status {
    // delete login from database
    let result: Option<bool> = db.query("
        if (
            DELETE login WHERE token = $tok RETURN id
        ) then true else false end;
    ").bind(("tok", &user.token))
        .await.and_then(|mut r| r.take(0))
        .expect("error executing logout query");

    let result = result.expect("error fetching logout query result");

    // return success or unauthorized error
    if result { Status::Ok } else { Status::Unauthorized }
}
