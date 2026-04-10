use crate::data::user_repository::Field;
use crate::domain::error;
use crate::domain::error::ValidationError;
use crate::infrastructure::jwt::JwtService;
use crate::presentation::dto::UserInfo;
use crate::{data::user_repository::UserRepository, presentation::dto};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use validator::Validate;

pub struct AuthService {
    user_repository: UserRepository,
}

impl AuthService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }

    pub async fn register_user(
        &self,
        mut data: dto::RegisterRequest,
        jwt_service: &JwtService,
    ) -> Result<dto::AuthResponse, error::UserError> {
        if let Err(e) = data.validate() {
            return Err(error::UserError::Validation(ValidationError {
                error: "data is invalid".into(),
                details: e.to_string(),
            }));
        }

        match self
            .user_repository
            .find_by_any(vec![
                Field::Email(data.email.clone()),
                Field::Username(data.username.clone()),
            ])
            .await
        {
            Ok(_) => {
                return Err(error::UserError::UserAlreadyExists(format!(
                    "user({}) with email {} is exist",
                    data.username, data.email
                )));
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => (),
                other_e => return Err(error::UserError::UnexpectedFailure(other_e.to_string())),
            },
        };

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(data.password.as_bytes(), &salt)
            .map_err(|e| error::UserError::UnexpectedFailure(e.to_string()))?
            .to_string();

        data.password = password_hash;

        let user = self
            .user_repository
            .create(data)
            .await
            .map_err(|e| error::UserError::UnexpectedFailure(e.to_string()))?;

        let token = match jwt_service.generate_token(user.id as u32, &user.username) {
            Ok(t) => t,
            Err(e) => {
                return Err(error::UserError::UnexpectedFailure(format!(
                    "internal error: {e}"
                )));
            }
        };

        Ok(dto::AuthResponse {
            token,
            user: UserInfo {
                id: user.id as u32,
                username: user.username,
                email: user.email,
            },
        })
    }

    pub async fn login_user(
        &self,
        data: dto::LoginRequest,
        jwt_service: &JwtService,
    ) -> Result<dto::AuthResponse, error::UserError> {
        if let Err(e) = data.validate() {
            return Err(error::UserError::Validation(ValidationError {
                error: "data is not valid".into(),
                details: e.to_string(),
            }));
        }
        let user = self
            .user_repository
            .find_by_username(&data.username)
            .await
            .map_err(|e| {
                error::UserError::NotFound(format!(
                    "user with name {} not found: {e}",
                    data.username
                ))
            })?;
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| error::UserError::UnexpectedFailure(e.to_string()))?;
        Argon2::default()
            .verify_password(data.password.as_bytes(), &parsed_hash)
            .map_err(|_| {
                error::UserError::InvalidCredentials(format!(
                    "failed to login user {}",
                    data.username
                ))
            })?;

        let token = match jwt_service.generate_token(user.id as u32, &user.username) {
            Ok(t) => t,
            Err(e) => {
                log::error!("login error: {e}");
                return Err(error::UserError::UnexpectedFailure("internal error".into()));
            }
        };

        let res = dto::AuthResponse {
            token,
            user: dto::UserInfo {
                id: user.id as u32,
                username: user.username,
                email: user.email,
            },
        };
        Ok(res)
    }
}
