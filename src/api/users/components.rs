//! User route components.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use crate::database::Id;

/// User update input body.
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UserIn {
    #[schema(example = "Alice")]
    #[validate(length(min = 2))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[schema(example = "user")]
    #[validate(custom(function = "validate_role"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

// User output body.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserOut {
    pub id: Id<String>,

    #[schema(example = "Alice")]
    pub name: String,

    #[schema(example = "user")]
    pub role: String,
}

/// Validate user role.
fn validate_role(value: &str) -> Result<(), ValidationError> {
    (value == "user" || value == "admin").then_some(())
        .ok_or(ValidationError::new("invalid_role"))
}
