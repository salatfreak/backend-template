//! Authentication route components.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::database::Id;

/// Registration input body.
#[derive(Serialize, Deserialize, Validate, ToSchema)]
pub struct RegisterIn {
    #[schema(example = "Alice")]
    #[validate(length(min = 2))]
    pub name: String,

    #[schema(example = "alice@example.com")]
    #[validate(email)]
    pub email: String,

    #[schema(example = "supersecret")]
    #[validate(length(min = 8))]
    pub password: String,
}

/// Registration confirmation input body.
#[derive(Deserialize, Validate, ToSchema)]
pub struct ConfirmIn {
    #[schema(example = "Ct6LXRBOcKKPdJAiiTKYb6NgQJWhxyLL")]
    #[validate(length(min = 32, max = 32))]
    pub token: String,
}

/// Login input body.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginIn {
    #[schema(example = "alice@example.com")]
    #[validate(email)]
    pub email: String,

    #[schema(example = "supersecret")]
    #[validate(length(min = 8))]
    pub password: String,
}

/// Login output body.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginOut {
    pub id: Id<String>,

    #[schema(example = "Alice")]
    pub name: String,

    #[schema(example = "user")]
    pub role: String,

    #[schema(example = "Ct6LXRBOcKKPdJAiiTKYb6NgQJWhxyLL")]
    pub token: String,
}
