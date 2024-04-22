//! Password reset route.

use rocket::{http::Status, post, serde::json::Json};
use validator::Validate;

use crate::{database::Database, mail::{Mail, Mailer}};
use super::{super::super::pow::POW, components::ResetIn};

#[utoipa::path(
    context_path = "/api/auth",
    request_body = ResetIn,
    responses(
        (status = 204, description = "Reset maybe successful"),
        (status = 402, description = "Invalid proof of work"),
    ),
    tag = "password reset",
)]

/// POST /api/auth/password/reset
///
/// Initiate password reset and send verification email if account with email
/// address exists and no password reset from within the last 30 minutes is
/// pending. The response will not expose whether this is the case to protect
/// the users' privacy. The request body needs to deliver a proof of work by
/// making sure the binary representation of its SHA512 hash begins with 16
/// zeros to achieve some protection against abuse by spammers. The example
/// request body will e.g. be accepted if `"nonce": 77761` is added as its last
/// field.
#[post("/password/reset", data = "<data>")]
pub async fn route(
    db: &Database, mail: &Mail, data: POW<Json<ResetIn>>,
) -> Status {
    // validate input
    if let Err(_) = data.validate() { return Status::UnprocessableEntity; }

    // query database to create password reset and return confirmation token
    let result: Option<String> = db.query("
        DELETE password_reset WHERE expires < time::now();

        let $uid = (
            SELECT id FROM ONLY user
            WHERE email = string::lowercase($email) AND id NOT IN (
                SELECT user FROM password_reset
            ).user LIMIT 1
        ).id;

        if $uid then (
            CREATE ONLY password_reset
            SET user = $uid, expires = time::now() + 30m
            RETURN token
        ).token end;
    ").bind(("email", &data.email))
        .await.and_then(|mut r| r.take(2))
        .expect("error executing password reset query");

    // spawn job for sending email if reset successfully initiated
    let mail: Mailer = mail.inner().clone();
    tokio::spawn(async move {
        if let Some(token) = result {
            let email = mail.template("reset-password", &[("token", &token)])
                .await.expect("error rendering email template");
            mail.send(&data.email, email).await.expect("error sending email");
        }
    });

    // return success status
    Status::NoContent
}
