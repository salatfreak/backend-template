use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use validator::Validate;

/// Password reset input body.
#[derive(Serialize, Deserialize, Validate, ToSchema)]
pub struct ResetIn {
    #[schema(example = "alice@example.com")]
    #[validate(email)]
    pub email: String,
}

/// Password reset confirmation input body.
#[derive(Deserialize, Validate, ToSchema)]
pub struct PasswordConfirmIn {
    #[schema(example = "Ct6LXRBOcKKPdJAiiTKYb6NgQJWhxyLL")]
    #[validate(length(min = 32, max = 32))]
    pub token: String,

    #[schema(example = "supersecret")]
    #[validate(length(min = 8))]
    pub password: String,
}

