//! User update route.

use rocket::{http::Status, patch, serde::json::Json};
use validator::Validate;

use crate::database::Database;
use super::{super::login::{Login, Owner, User}, components::{UserIn, UserOut}};

#[utoipa::path(
    context_path = "/api/users",
    request_body = UserIn,
    responses(
        (status = 200, description = "Updated user", body = UserOut),
        (status = 401, description = "Invalid login session"),
        (status = 403, description = "Insufficient privileges"),
    ),
    security(("login" = [])),
    tag = "users",
)]

#[patch("/<id>", data = "<data>")]
pub async fn route(
    user: Login<User>, db: &Database, id: &str, data: Json<UserIn>,
) -> Result<Json<UserOut>, Status> {
    // validate input
    data.validate().map_err(|_| Status::UnprocessableEntity)?;

    // only allow updating own info if not owner
    (user.is(Owner) || user.id == id)
        .then_some(()).ok_or(Status::Forbidden)?;

    // restrict updating role to owner
    (user.is(Owner) || data.role.is_none())
        .then_some(()).ok_or(Status::Forbidden)?;

    // query database to update user
    let users: Option<UserOut> = db.update(("user", id))
        .merge(data.into_inner())
        .await.expect("error updating user");

    // return users or not found status
    users.map(Json).ok_or(Status::NotFound)
}
