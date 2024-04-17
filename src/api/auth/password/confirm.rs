//! Password reset confirmation route.

use rocket::{http::Status, post, serde::json::Json};
use validator::Validate;

use crate::database::Database;
use super::components::PasswordConfirmIn;

#[utoipa::path(
    context_path = "/api/auth",
    request_body = PasswordConfirmIn,
    responses(
        (status = 204, description = "Reset successful"),
        (status = 404, description = "Reset token not found"),
    ),
    tag = "password reset",
)]

/// POST /api/auth/password/confirm
///
/// Confirm password reset that has been initiated within the past 30 minutes.
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

    // return success or not found status
    if success { Status::NoContent } else { Status::NotFound }
}
