//! Hierarchy of API routes.

use rocket::{error, fairing::AdHoc};
use surrealdb::{engine::any::Any, Surreal};

use crate::config::APIConfig;

pub mod login;
pub mod auth;
pub mod users;

/// Mount API routes to the rocket instance.
pub fn mount(config: APIConfig) -> AdHoc {
    AdHoc::try_on_ignite("API Routes", |rocket| async move {
        // update owner user
        if let Some(owner) = config.owner {
            // split value into email and password
            let (email, hash) = match owner.split_once(":") {
                Some(pair) => pair,
                None => {
                    error!("owner must be colon separated email and password");
                    return Err(rocket);
                },
            };

            // get database connection managed by rocket
            let db = match rocket.state::<Surreal<Any>>() {
                Some(db) => db,
                None => {
                    error!("Error getting database");
                    return Err(rocket);
                }
            };

            // create or update owner user
            if let Err(err) = update_owner(db, email, hash).await {
                error!("SurrealDB: {:?}", err);
                return Err(rocket);
            }
        }

        // mount API routes
        Ok(rocket
            .mount("/api/auth", auth::routes())
            .mount("/api/users", users::routes())
        )
    })
}

/// Create or update owner user.
async fn update_owner(
    db: &Surreal<Any>, email: &str, hash: &str,
) -> Result<(), surrealdb::Error> {
    db.query("
        let $uid = (
            UPDATE ONLY user SET password = $hash, role = 'owner'
            WHERE email = $email RETURN id
        ).id;

        if !$uid {
            CREATE user SET
                name = 'Owner', email = $email,
                password = $hash, role = 'owner';
        };
    ").bind(("email", email)).bind(("hash", hash)).await?;
    Ok(())
}
