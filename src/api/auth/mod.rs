use rocket::{routes, Route};

pub mod login;
pub mod logout;

pub fn routes() -> Vec<Route> {
    routes![login::route, logout::route]
}

