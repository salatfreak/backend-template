//! SurrealDB database integration.

use rocket::{error, fairing::AdHoc, State};
use surrealdb::{
    engine::remote::ws::{Ws, Client},
    opt::auth::Root,
    Surreal,
};

mod id;
pub mod login;

pub use id::Id;

use crate::config::DatabaseConfig;

/// Database type alias for abbreviation in route handlers.
pub type Database = State<Surreal<Client>>;

/// Create and mount database to the rocket instance.
pub fn mount(config: DatabaseConfig) -> AdHoc {
    AdHoc::try_on_ignite("Mount SurrealDB", |rocket| async move {
        // create database
        let result = create(
            &config.host, &config.username, &config.password,
            &config.namespace, &config.database,
        ).await;

        // handle errors
        let db = match result{
            Ok(db) => db,
            Err(err) => {
                error!("SurrealDB: {:?}", err);
                return Err(rocket)
            }
        };

        // add database to rocket instance
        Ok(rocket.manage(db))
    })
}

/// Connect and log in to database.
async fn create(
    address: &str,
    username: &str, password: &str,
    namespace: &str, database: &str,
) -> Result<Surreal<Client>, surrealdb::Error> {
    let db = Surreal::new::<Ws>(address).await?;
    db.signin(Root { username, password }).await?;
    db.use_ns(namespace).use_db(database).await?;
    Ok(db)
}
