use std::future::{Ready, ready};
use std::sync::Arc;

use crate::infrastructure::jwt::JwtService;
use actix_web::dev::ServiceRequest;
use actix_web::{FromRequest, HttpMessage, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;

#[derive(Debug, Clone)]
pub struct AuthentificatedUser {
    pub user_id: u32,
    pub username: String,
}

impl AuthentificatedUser {
    pub fn get_id(&self) -> u32 {
        self.user_id
    }

    pub fn get_name(&self) -> &str {
        &self.username
    }
}

impl FromRequest for AuthentificatedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(user) = req.extensions().get::<AuthentificatedUser>() {
            ready(Ok(user.clone()))
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized(
                "missing authentificated user",
            )))
        }
    }
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let token = credentials.token();
    let Some(jwt_s) = req.app_data::<web::Data<Arc<JwtService>>>() else {
        log::error!("jwt service not found");
        return Err((
            actix_web::error::ErrorInternalServerError("jwt service no found"),
            req,
        ));
    };
    let claims = match jwt_s.verify_token(&token) {
        Ok(c) => c,
        Err(e) => {
            log::error!("jwt verify error: {e}");
            return Err((
                actix_web::error::ErrorBadRequest(format!("bad token: {e}")),
                req,
            ));
        }
    };
    req.extensions_mut().insert(AuthentificatedUser {
        user_id: claims.get_user_id(),
        username: claims.get_user_name().to_string(),
    });
    Ok(req)
}
