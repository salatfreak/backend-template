//! Hierarchy of API routes.

use rocket::fairing::AdHoc;

pub mod auth;
pub mod users;

pub fn mount() -> AdHoc {
    AdHoc::on_ignite("API Routes", |rocket| async {
        rocket
            .mount("/api/auth", auth::routes())
            .mount("/api/users", users::routes())
    })
}
