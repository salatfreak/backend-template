//! Embeding and running of database migrations.

use include_dir::{include_dir, Dir, File};
use rocket::info;
use serde::Deserialize;
use sha2::{Sha256, Digest};
use std::{iter::zip, str};
use surrealdb::{engine::remote::ws::Client, Surreal};

use super::Id;

/// Statically loaded migrations.
static MIGRATION_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/migrations");

/// Migration to be constructed from migration file.
#[derive(Debug)]
struct Migration {
    name: String,
    content: String,
    hash: String,
}

impl TryFrom<&File<'_>> for Migration {
    type Error = Error;

    fn try_from(file: &File<'_>) -> Result<Self, Self::Error> {
        // extract name and content from file
        let name = file.path().with_extension("").file_name()
            .and_then(|n| n.to_str()).ok_or(Error::DirectoryTree)?.into();
        let content = file.contents_utf8().ok_or(Error::DirectoryTree)?.into();

        // calculate content hash
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        // construct and return migration
        Ok(Self { name, content, hash })
    }
}

/// Migration to be loaded from the database.
#[derive(Debug, Deserialize)]
struct MigrationHash {
    id: Id<String>,
    hash: String,
}

/// Migration error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error in directory tree")]
    DirectoryTree,

    #[error("database error")]
    Database { #[from] source: surrealdb::Error },

    #[error("database expected newer program version")]
    Outdated,

    #[error("expected \"{expected}\" but found \"{database}\" in database")]
    Inconsistency { expected: String, database: String },

    #[error("hash mismatch for migration \"{name}\"")]
    HashMismatch { name: String },
}

/// Apply all open migrations to the specified database.
pub async fn apply(db: &Surreal<Client>) -> Result<(), Error> {
    // load current migrations and database state
    let migrations = parse_migrations(&MIGRATION_DIR)?;
    let state: Vec<MigrationHash> = db.query("
        SELECT * FROM database_migration ORDER BY id
    ").await?.take(0)?;

    // check database state consistency
    if state.len() > migrations.len() {
        return Err(Error::Outdated)
    }

    for (st, mig) in zip(&state, &migrations) {
        // handle inconsitent migration names
        if st.id != mig.name {
            return Err(Error::Inconsistency {
                expected: mig.name.clone(), database: st.id.0.clone(),
            });
        }

        // handle hash missmatch
        if st.hash != mig.hash {
            return Err(Error::HashMismatch { name: mig.name.clone() });
        }
    }

    // apply new migrations
    for mig in migrations[state.len()..].into_iter() {
        info!("Applying migration \"{}\"", mig.name);

        db.query(&mig.content).query("
            CREATE type::thing('database_migration', $name) SET hash = $hash
        ").bind(("hash", &mig.hash)).bind(("name", &mig.name)).await?;
    }

    // return success
    Ok(())
}

/// Parse directory into migrations.
fn parse_migrations(dir: &Dir<'_>) -> Result<Vec<Migration>, Error> {
    // get sorted .surql files
    let mut files: Vec<&File<'_>> = dir.files().filter(|f|
            f.path().extension().and_then(|s| s.to_str()) == Some("surql")
        ).collect();
    files.sort_by(|a, b| a.path().cmp(b.path()));

    // parse files into migrations
    files.into_iter().map(|f| f.try_into()).collect()
}
