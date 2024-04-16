use rocket::{routes, Route};

pub mod components;
pub mod register;
pub mod confirm;
pub mod login;
pub mod logout;
pub mod password;

pub fn routes() -> Vec<Route> {
    routes![
        register::route, confirm::route,
        login::route, logout::route,
        password::reset::route, password::confirm::route,
    ]
}

