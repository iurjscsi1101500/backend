use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use email::ResendMailer;
use rand::rngs::OsRng;
use rocket::futures::TryFutureExt;
use rocket::http::uri::Absolute;
use rocket::serde::json::Json;
use schemars::_serde_json::json;
use sea_orm::prelude::Uuid;
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter,
    TransactionTrait,
};
use shared::responses::error::{AppError, Error};

use crate::domain::entities::{EmailEntity, PasswordEntity, UserEntity};
use crate::presentation::dto::create_user::{CreateUserDTO, UserCreatedDTO};

#[derive(Debug)]
pub enum UserCreationError {
    DuplicateUsername,
    DuplicateEmail,
    PasswordHashingError(String),
    DatabaseError(sea_orm::DbErr),
    SendEmailError(String),
}

impl From<UserCreationError> for Error {
    fn from(error: UserCreationError) -> Self {
        match error {
            UserCreationError::DuplicateUsername => AppError::BadRequest("Username already exists".to_string()).into(),
            UserCreationError::DuplicateEmail => AppError::BadRequest("Email already exists".to_string()).into(),
            UserCreationError::PasswordHashingError(msg) => AppError::InternalError(msg).into(),
            UserCreationError::DatabaseError(err) => AppError::InternalError(err.to_string()).into(),
            UserCreationError::SendEmailError(msg) => AppError::InternalError(msg).into(),
        }
    }
}

impl From<sea_orm::DbErr> for UserCreationError {
    fn from(err: sea_orm::DbErr) -> Self { UserCreationError::DatabaseError(err) }
}

pub async fn action(
    user: CreateUserDTO, conn: &DatabaseConnection, mailer: ResendMailer, base_api_url: String,
) -> Result<Json<UserCreatedDTO>, Error> {
    check_existing_user(&user, conn).await?;

    let txn = conn.begin().await?;

    let user_db = create_user_record(&user, &txn).await?;
    let email_db = create_email_record(&user, user_db.last_insert_id, &txn).await?;
    let password_db = create_password_record(&user, user_db.last_insert_id, &txn).await?;

    let activate_url = format!("{}/user/activate?token={}&user_id{}", base_api_url, user_db.last_insert_id, email_db.1);
    let activate_url = Absolute::parse(&activate_url).map_err(|e| UserCreationError::SendEmailError(e.to_string()))?;

    mailer
        .send_email(
            vec![&user.email],
            "Welcome to the Meisaku!",
            "sign_up",
            json!({ "user_name": user.username, "activation_url": activate_url.to_string() }),
        )
        .map_err(|e| UserCreationError::SendEmailError(e.to_string()))
        .await?;

    txn.commit().await?;

    Ok(Json(UserCreatedDTO {
        id: user_db.last_insert_id,
        email_id: email_db.0.last_insert_id,
        password_id: password_db.last_insert_id,
    }))
}

async fn check_existing_user(user: &CreateUserDTO, conn: &impl ConnectionTrait) -> Result<(), UserCreationError> {
    let user_exist = UserEntity::Entity::find()
        .filter(UserEntity::Column::Username.eq(&user.username))
        .one(conn)
        .await?;

    if user_exist.is_some() {
        return Err(UserCreationError::DuplicateUsername);
    }

    let email_exist = EmailEntity::Entity::find()
        .filter(EmailEntity::Column::Key.eq(&user.email))
        .one(conn)
        .await?;

    if email_exist.is_some() {
        return Err(UserCreationError::DuplicateEmail);
    }

    Ok(())
}

async fn create_user_record(
    user: &CreateUserDTO, txn: &DatabaseTransaction,
) -> Result<sea_orm::InsertResult<UserEntity::ActiveModel>, UserCreationError> {
    Ok(UserEntity::Entity::insert(UserEntity::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        username: ActiveValue::Set(user.username.clone()),
        ..Default::default()
    })
    .exec(txn)
    .await?)
}

async fn create_email_record(
    user: &CreateUserDTO, user_id: Uuid, txn: &DatabaseTransaction,
) -> Result<(sea_orm::InsertResult<EmailEntity::ActiveModel>, Uuid), UserCreationError> {
    let token_activation = Uuid::new_v4();

    Ok((
        EmailEntity::Entity::insert(EmailEntity::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            key: ActiveValue::Set(user.email.clone()),
            active: ActiveValue::Set(false),
            activation_token: ActiveValue::Set(token_activation),
            user_id: ActiveValue::Set(user_id),
            ..Default::default()
        })
        .exec(txn)
        .await?,
        token_activation,
    ))
}

async fn create_password_record(
    user: &CreateUserDTO, user_id: Uuid, txn: &DatabaseTransaction,
) -> Result<sea_orm::InsertResult<PasswordEntity::ActiveModel>, UserCreationError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(user.password.as_bytes(), &salt)
        .map_err(|e| UserCreationError::PasswordHashingError(e.to_string()))?
        .to_string();

    Ok(PasswordEntity::Entity::insert(PasswordEntity::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        user_id: ActiveValue::Set(user_id),
        hash: ActiveValue::Set(password_hash),
        salt: ActiveValue::Set(salt.to_string()),
        ..Default::default()
    })
    .exec(txn)
    .await?)
}
