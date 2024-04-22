//! Registration route.

use rocket::{http::Status, post, serde::json::Json};
use validator::Validate;

use crate::{database::Database, mail::{Mail, Mailer}};
use super::{super::pow::POW, components::RegisterIn};

#[utoipa::path(
    context_path = "/api/auth",
    request_body = RegisterIn,
    responses(
        (status = 204, description = "Registration maybe successful"),
        (status = 402, description = "Invalid proof of work"),
    ),
    tag = "authentication",
)]

/// POST /api/auth/register
///
/// Initiate new account registration and send verification email if email
/// address is not already registered. The response will not expose whether
/// that is the case to protect the users' privacy. Unconfirmed registrations 
/// expire after 30 minutes.
///
/// The request body needs to deliver a proof of work by making sure the
/// binary representation of its SHA512 hash begins with 16 zeros to achieve
/// some protection against abuse by spammers. The example request body will
/// e.g. be accepted if `"nonce": 85223` is added as its last field.
#[post("/register", data = "<data>")]
pub async fn route(
    db: &Database, mail: &Mail, data: POW<Json<RegisterIn>>
) -> Status {
    // validate input
    if let Err(_) = data.validate() { return Status::UnprocessableEntity; }

    // query database to create registration and return confirmation token
    let result: Option<String> = db.query("
        DELETE registration WHERE expires < time::now();

        let $existing = array::concat((
            SELECT id FROM registration
            WHERE data.email = string::lowercase($data.email)
        ), (
            SELECT id FROM user
            WHERE email = string::lowercase($data.email)
        ));

        if !$existing then (
            CREATE ONLY registration
            SET data = $data, expires = time::now() + 30m
            RETURN token
        ).token end;
    ").bind(("data", &**data))
        .await.and_then(|mut r| r.take(2))
        .expect("error executing registration query");

    // spawn job for sending email if registration successful
    let mail: Mailer = mail.inner().clone();
    tokio::spawn(async move {
        if let Some(token) = result {
            let email = mail.template("verify-account", &[("token", &token)])
                .await.expect("error rendering email template");
            mail.send(&data.email, email).await.expect("error sending email");
        }
    });

    // return success status
    Status::NoContent
}
