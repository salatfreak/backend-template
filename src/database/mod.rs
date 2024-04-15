//! SurrealDB database integration.


use rocket::{error, fairing::AdHoc, State};
use surrealdb::{
    engine::any::{self, Any},
    opt::auth::Root,
    Surreal,
};

mod migrations;
mod id;

pub use id::Id;

use crate::config::DatabaseConfig;

/// Database type alias for abbreviation in route handlers.
pub type Database = State<Surreal<Any>>;

/// Create and mount database to the rocket instance.
pub fn mount(config: DatabaseConfig) -> AdHoc {
    AdHoc::try_on_ignite("Mount SurrealDB", |rocket| async move {
        // assemble credentials only if both given and not in-memory
        let DatabaseConfig { address, username, password, .. } = config;
        let credentials = match (address.as_str(), &username, &password) {
            ("memory", _, _) => None,
            (_, Some(usr), Some(pass)) => Some((usr.as_str(), pass.as_str())),
            _ => None,
        };

        // create database
        let DatabaseConfig { namespace, database, .. } = config;
        let result = create(&address, credentials, &namespace, &database).await;

        // handle errors
         let db = match result{
             Ok(db) => db,
             Err(err) => { error!("SurrealDB: {:?}", err); return Err(rocket); }
        };

        // apply migrations
        if let Err(err) = migrations::apply(&db).await {
            error!("SurrealDB migrations: {:?}", err);
            return Err(rocket);
        }

        // add database to rocket instance
        Ok(rocket.manage(db))
    })
}

/// Connect and optionally log in to database.
async fn create(
    address: &str, credentials: Option<(&str, &str)>,
    namespace: &str, database: &str,
) -> Result<Surreal<Any>, surrealdb::Error> {
    let db = any::connect(address).await?;
    if let Some((username, password)) = credentials {
        db.signin(Root { username, password }).await?;
    }
    db.use_ns(namespace).use_db(database).await?;
    Ok(db)
}
