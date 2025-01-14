use rocket::serde::{Deserialize, Serialize};
use rocket_validation::Validate;
use sea_orm::prelude::Uuid;

/// # Active Email DTO
///
/// The data transfer object for activating an email.
#[derive(schemars::JsonSchema, Serialize, Deserialize, Validate, Clone)]
#[serde(crate = "rocket::serde")]
#[schemars(deny_unknown_fields)]
pub struct ActiveEmailDTO {
    /// The token to activate the email
    ///
    /// This token is generated by the server and sent to the user's email.
    #[schemars(length(min = 32, max = 32))]
    pub token: Uuid,
    /// User id
    ///
    /// The id of the user that the email is being activated for.
    #[schemars(length(min = 32, max = 32))]
    pub user_id: Uuid,
}
