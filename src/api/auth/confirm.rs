//! Confirm route.

use rocket::{http::Status, post, serde::json::Json};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::{database::Database, mail::Mail};

use super::login::LoginOut;

/// Input data schema.
#[derive(Deserialize, Validate, ToSchema)]
pub struct ConfirmIn {
    #[schema(example = "Ct6LXRBOcKKPdJAiiTKYb6NgQJWhxyLL")]
    #[validate(length(min = 32, max = 32))]
    pub token: String,
}

/// Database response type.
#[derive(Deserialize)]
struct DbOutput {
    email_address: String,
    login: LoginOut,
}

#[utoipa::path(
    context_path = "/api/auth",
    request_body = ConfirmIn,
    responses((
        status = 200, description = "Successful registration",
        body = LoginOut
    ), (
        status = 404, description = "Registration token not found",
    )),
    tag = "authentication",
)]

#[post("/confirm", data = "<data>")]
pub async fn route(
    db: &Database, mail: &Mail, data: Json<ConfirmIn>
) -> Result<Json<LoginOut>, Status> {
    // validate input
    data.validate().map_err(|_| Status::UnprocessableEntity)?;

    // query database to confirm registration and create login session
    let result: Option<DbOutput> = db.query("
        DELETE registration WHERE expires < time::now();

        let $data = (
            DELETE registration WHERE token = $tok RETURN BEFORE
        )[0].data;

        if $data {
            let $uid = (CREATE ONLY user CONTENT $data RETURN id).id;

            let $login = (
                CREATE ONLY login SET user = $uid RETURN
                user.id AS id, user.name AS name, user.role AS role, token
            );

            { \"email_address\": $data.email, \"login\": $login };
        };
    ").bind(("tok", &data.token))
        .await.and_then(|mut r| r.take(2))
        .expect("error executing registration query");

    // extract results or return not found
    let DbOutput { email_address, login } = result.ok_or(Status::NotFound)?;

    // send email if confirmation successful
    let email = mail.template("confirm-account", &[("name", &login.name)])
        .await.expect("error rendering email template");
    mail.send(&email_address, email).await.expect("error sending email");

    // return JSON response
    Ok(Json(login))
}
