//! Routes for OpenAPI specification and RapiDoc UI.

use rocket::{
    error, fairing::AdHoc, http::Method, response::content::RawJson,
    route::{Handler, Outcome},
    Data, Request, Route,
};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

use crate::{api, database};

/// URI where OpenAPI JSON is being served.
pub const OPENAPI_URI: &str = "/api-docs/openapi.json";

/// Mount OpenAPI and RapiDoc documentation routes.
pub fn mount() -> AdHoc {
    AdHoc::try_on_ignite("RapiDoc Documentation", |rocket| async {
        let json = match ApiDoc::openapi().to_json() {
            Ok(json) => json,
            Err(_) => { error!("invalid OpenAPI JSON"); return Err(rocket); }
        };

        Ok(rocket
            .mount(OPENAPI_URI, RawJsonHandler(json))
            .mount("/", RapiDoc::new(OPENAPI_URI).path("/doc"))
        )
    })
}

/// Documentation object.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Backend API",
        license(name = "GNU General Public License v3.0"),
        description =
            "Interactive API documentation.",
    ),
    paths(api::index, api::get, api::create),
    components(schemas(api::User, api::NewUser, database::Id<String>)),
)]
struct ApiDoc;

/// Raw JSON handler for serving rendered OpenAPI documentation.
#[derive(Clone)]
struct RawJsonHandler(String);

#[rocket::async_trait]
impl Handler for RawJsonHandler {
    async fn handle<'r>(
        &self, req: &'r Request<'_>, _: Data<'r>,
    ) -> Outcome<'r> {
        Outcome::from(req, RawJson(self.0.clone()))
    }
}

impl From<RawJsonHandler> for Vec<Route> {
    fn from(value: RawJsonHandler) -> Self {
        vec![Route::new(Method::Get, "/", value)]
    }
}
