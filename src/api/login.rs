//! User login request guard with role discrimination.

use core::fmt;
use std::marker::PhantomData;

use rocket::{error, http::Status, request::{FromRequest, Outcome}, Request};
use serde::Deserialize;

use crate::database::{Database, Id};

/// Generic login request guard.
#[derive(Debug, Deserialize)]
pub struct Login<R: Role> {
    pub id: Id<String>,
    pub name: String,
    pub role: String,
    pub token: String,
    #[serde(skip)]
    phantom: PhantomData<R>,
}

impl<R: Role> Login<R> {
    /// Check if login satisfied associated role.
    pub fn satisfied(&self) -> bool {
        R::satisfied(self)
    }

    /// Check if login satisfies specified role.
    pub fn is<S: Role>(&self, _: S) -> bool {
        S::satisfied(self)
    }
}

/// Login server error enum.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error receiving database from request")]
    DatabaseGuard,

    #[error("SurrealDB error")]
    SurrealDB { #[from] source: surrealdb::Error }
}

/// Request guard implementation for session and role validation.
#[rocket::async_trait]
impl <'r, R: Role> FromRequest<'r> for Login<R> {
    type Error = Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // extract token from header
        let token = req.headers()
            .get_one("authorization")
            .and_then(|value| value.split_once(" "))
            .and_then(|kv| match kv { ("apikey", k) => Some(k), _ => None });

        let token = match token {
            Some(token) => token,
            None => return Outcome::Forward(Status::Unauthorized),
        };

        // get database from request
        let db = match req.guard::<&Database>().await {
            Outcome::Success(db) => db,
            Outcome::Forward(status) | Outcome::Error((status, _)) =>
                return Outcome::Error((status, Error::DatabaseGuard)),
        };

        // get session from database
        let result: Result<Option<Login<R>>, surrealdb::Error> = db.query("
            SELECT user.id AS id, user.name AS name, user.role AS role, token
            FROM ONLY login WHERE token = $tok LIMIT 1
        ").bind(("tok", token)).await.and_then(|mut r| r.take(0));

        // handle database errors and invalid session
        let login = match result {
            Ok(Some(login)) => login,
            Ok(None) => return Outcome::Forward(Status::Unauthorized),
            Err(err) => return Outcome::Error(
                (Status::InternalServerError, err.into())
            ),
        };

        // handle user role
        match login.satisfied() {
            true => Outcome::Success(login),
            false => Outcome::Forward(Status::Forbidden),
        }
    }
}

/// Trait for evaluating user role membership.
pub trait Role: Sized + fmt::Debug {
    fn satisfied<R: Role>(login: &Login<R>) -> bool;
}

/// Login role for all users.
#[derive(Debug)]
pub struct User;
impl Role for User {
    fn satisfied<R: Role>(_: &Login<R>) -> bool {
        true
    }
}

/// Login role for all admin users.
#[derive(Debug)]
pub struct Admin;
impl Role for Admin {
    fn satisfied<R: Role>(login: &Login<R>) -> bool {
        login.role == "admin" || login.role == "owner"
    }
}

/// Login role for system owner.
#[derive(Debug)]
pub struct Owner;
impl Role for Owner {
    fn satisfied<R: Role>(login: &Login<R>) -> bool {
        login.role == "owner"
    }
}
