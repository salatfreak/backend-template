//! Password reset route.

use rocket::{http::Status, post, serde::json::Json};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{database::Database, mail::{Mail, Mailer}};

/// Input data schema.
#[derive(Serialize, Deserialize, Validate, ToSchema)]
pub struct ResetIn {
    #[schema(example = "alice@example.com")]
    #[validate(email)]
    pub email: String,
}

#[utoipa::path(
    context_path = "/api/auth",
    request_body = ResetIn,
    responses(
        (status = 200, description = "Reset maybe successful"),
    ),
)]

#[post("/password/reset", data = "<data>")]
pub async fn route(
    db: &Database, mail: &Mail, data: Json<ResetIn>
) -> Status {
    // validate input
    if let Err(_) = data.validate() { return Status::UnprocessableEntity; }

    // query database to create password reset and return confirmation token
    let result: Option<String> = db.query("
        DELETE password_reset WHERE expires < time::now();

        let $uid = (
            SELECT id FROM ONLY user
            WHERE email = string::lowercase($email)
            LIMIT 1
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
    Status::Ok
}
