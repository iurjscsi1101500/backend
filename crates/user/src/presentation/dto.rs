use rocket::serde::{Deserialize, Serialize};
use rocket_validation::Validate;

/// # Create user
///
/// Data transfer object for creating a user.
#[derive(schemars::JsonSchema, Serialize, Deserialize, Validate)]
#[serde(crate = "rocket::serde")]
#[schemars(deny_unknown_fields)]
pub struct CreateUserDTO {
    /// The username of the user. Must be unique.
    #[schemars(length(min = 3, max = 16))]
    #[validate(length(min = 3, max = 16, message = "Username must be between 3 and 16 characters long."))]
    pub username: String,
    /// The email of the user. Must be unique.
    #[schemars(email)]
    #[validate(email(message = "Email must be a valid email address."))]
    pub email: String,
    /// The password of the user.
    #[schemars(length(min = 6, max = 32))]
    #[validate(length(min = 6, max = 32, message = "Password must be between 6 and 32 characters long."))]
    pub password: String,
}

/// # User created
///
/// Data of the user created. The id is generated by the database and returned
/// to the user.
///
/// To get the user's information, the user must query the user
/// endpoint with the id or use the auth endpoint to get the user's information.
#[derive(schemars::JsonSchema, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UserCreatedDTO {
    /// The id of the user. This id is generated by the database.
    pub id: i32,
    /// The username of the user.
    pub username: String,
    /// The email of the user.
    pub email: String,
}
