use rocket::{routes, Route};

pub mod register;
pub mod confirm;
pub mod login;
pub mod logout;

pub fn routes() -> Vec<Route> {
    routes![register::route, confirm::route, login::route, logout::route]
}

