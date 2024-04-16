//! Registration route.

use rocket::{http::Status, post, serde::json::Json};
use validator::Validate;

use crate::{database::Database, mail::{Mail, Mailer}};
use super::components::RegisterIn;

#[utoipa::path(
    context_path = "/api/auth",
    request_body = RegisterIn,
    responses(
        (status = 204, description = "Registration maybe successful"),
    ),
    tag = "authentication",
)]

#[post("/register", data = "<data>")]
pub async fn route(
    db: &Database, mail: &Mail, data: Json<RegisterIn>
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
    ").bind(("data", &*data))
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
