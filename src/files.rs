//! Static file routes and error handlers.

use rocket::{
    catcher, fairing::AdHoc,
    fs::{FileServer, NamedFile},
    http::{Method, Status},
    outcome::IntoOutcome, response::Responder,
    route::{self, Outcome},
    Catcher, Data, Request, Response, Route,
};
use std::path::PathBuf;

/// Mount static file routes and error handlers to rocket instance.
pub fn mount(path: PathBuf) -> AdHoc {
    let index = path.join("index.html");
    let assets = path.join("assets");
    let not_found = path.join("not_found.html");
    AdHoc::on_ignite("Static File Routes", |rocket| async {
        rocket
            .mount("/", vec![Route::new(Method::Get, "/", FileHandler(index))])
            .mount("/assets", FileServer::from(assets).rank(0))
            .register("/", vec![Catcher::new(404, FileHandler(not_found))])
    })
}

/// File handler for router and catcher.
#[derive(Clone)]
struct FileHandler(PathBuf);

#[rocket::async_trait]
impl route::Handler for FileHandler {
    async fn handle<'r>(
        &self, req: &'r Request<'_>, data: Data<'r>
    ) -> Outcome<'r> {
        NamedFile::open(&self.0).await.respond_to(req)
            .or_forward((data, Status::NotFound))
    }
}

#[rocket::async_trait]
impl catcher::Handler for FileHandler {
    async fn handle<'r>(
        &self, status: Status, req: &'r Request<'_>
    ) -> catcher::Result<'r> {
        NamedFile::open(&self.0).await.respond_to(req)
            .and_then(|resp| Response::build_from(resp).status(status).ok())
    }
}
