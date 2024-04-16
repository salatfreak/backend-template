//! User routes.

use rocket::{routes, Route};

pub mod components;
pub mod index;
pub mod show;
pub mod update;
pub mod destroy;

/// Assemble user routes.
pub fn routes() -> Vec<Route> {
    routes![index::route, show::route, update::route, destroy::route]
}
