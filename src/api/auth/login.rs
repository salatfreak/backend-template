//! Login route.

use rocket::{http::Status, post, serde::json::Json};
use validator::Validate;

use crate::database::Database;
use super::components::{LoginIn, LoginOut};

#[utoipa::path(
    context_path = "/api/auth",
    request_body = LoginIn,
    responses(
        (status = 200, description = "Login successful", body = LoginOut),
        (status = 401, description = "Invalid login data"),
    ),
    tag = "authentication",
)]

/// POST /api/auth/login
///
/// Create login session for already registered user.
#[post("/login", data = "<data>")]
pub async fn route(
    db: &Database, data: Json<LoginIn>,
) -> Result<Json<LoginOut>, Status> {
    // validate input
    data.validate().map_err(|_| Status::UnprocessableEntity)?;

    // query database to validate credentials and create login
    let result: Option<LoginOut> = db.query("
        let $user = (
            SELECT id FROM ONLY user
            WHERE email = $email 
                AND crypto::argon2::compare(password, $pass)
            LIMIT 1
        );

        if $user then (
            CREATE ONLY login SET user = $user.id
            RETURN user AS id, user.name AS name, user.role AS role, token
        ) end;
    ").bind(("email", &data.email)).bind(("pass", &data.password))
        .await.and_then(|mut r| r.take(1))
        .expect("error executing login query");

    // return json response or unauthorized error
    result.map(Json).ok_or(Status::Unauthorized)
}
