//! Password reset confirm route.

use rocket::{http::Status, post, serde::json::Json};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::database::Database;

/// Input data schema.
#[derive(Deserialize, Validate, ToSchema)]
pub struct PasswordConfirmIn {
    #[schema(example = "Ct6LXRBOcKKPdJAiiTKYb6NgQJWhxyLL")]
    #[validate(length(min = 32, max = 32))]
    pub token: String,

    #[schema(example = "supersecret")]
    #[validate(length(min = 8))]
    pub password: String,
}

#[utoipa::path(
    context_path = "/api/auth",
    request_body = PasswordConfirmIn,
    responses(
        ( status = 200, description = "Reset successful" ),
        ( status = 404, description = "Reset token not found" ),
    ),
    tag = "password reset",
)]

#[post("/password/confirm", data = "<data>")]
pub async fn route(db: &Database, data: Json<PasswordConfirmIn>) -> Status {
    // validate input
    if let Err(_) = data.validate() { return Status::UnprocessableEntity; }

    // query database to confirm registration
    let result: Option<bool> = db.query("
        DELETE password_reset WHERE expires < time::now();

        let $uid = (
            DELETE password_reset WHERE token = $tok RETURN BEFORE
        )[0].user;

        if $uid {
            DELETE login WHERE user = $uid;
            UPDATE $uid SET password = crypto::argon2::generate($pass);
            true;
        } else {
            false;
        };
    ").bind(("tok", &data.token)).bind(("pass", &data.password))
        .await.and_then(|mut r| r.take(2))
        .expect("error executing password reset confirmation query");

    let success = result
        .expect("error fetching data from password reset confirmation query");

    // Return success or not found status
    if success { Status::Ok } else { Status::NotFound }
}
