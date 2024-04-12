//! Login route.

use rocket::{http::Status, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::database::{Database, Id};

/// Input data schema.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginIn {
    #[schema(example = "alice@example.com")]
    #[validate(email)]
    pub email: String,

    #[schema(example = "supersecret")]
    #[validate(length(min = 8))]
    pub password: String,
}

/// Output data schema.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginOut {
    pub id: Id<String>,

    #[schema(example = "Alice")]
    pub name: String,

    #[schema(example = "user")]
    pub role: String,

    #[schema(example = "Ct6LXRBOcKKPdJAiiTKYb6NgQJWhxyLL")]
    pub token: String,
}

#[utoipa::path(
    context_path = "/api/auth",
    request_body = LoginIn,
    responses(
        (status = 200, description = "Login successful", body = LoginOut),
        (status = 401, description = "Invalid login data"),
    ),
)]

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
    result.map(|r| Json(r)).ok_or(Status::Unauthorized)
}
