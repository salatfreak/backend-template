//! User listing route.

use rocket::{get, serde::json::Json};

use crate::database::Database;
use super::{super::login::{Admin, Login}, components::UserOut};

#[utoipa::path(
    context_path = "/api/users",
    responses(
        (status = 200, description = "List of users", body = Vec<UserOut>),
        (status = 401, description = "Invalid login session"),
        (status = 403, description = "Insufficient privileges"),
    ),
    security(("login" = [])),
    tag = "users",
)]

#[get("/")]
pub async fn route(_user: Login<Admin>, db: &Database) -> Json<Vec<UserOut>> {
    let users: Vec<UserOut> = db.select("user").await
        .expect("error retrieving users");
    Json(users)
}
