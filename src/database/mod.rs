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

/// Database type alias for abbreviation in route handlers.
pub type Database = State<Surreal<Client>>;

/// Create and mount database to the rocket instance.
pub fn mount(
    address: &str,
    username: &str,
    password: &str,
    namespace: &str,
    database: &str,
) -> AdHoc {
    let addr = address.to_owned();
    let user = username.to_owned();
    let pass = password.to_owned();
    let ns = namespace.to_owned();
    let db = database.to_owned();

    AdHoc::try_on_ignite("Mount SurrealDB", |rocket| async move {
        // create database
        let db = match create(&addr, &user, &pass, &ns, &db).await {
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
