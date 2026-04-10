use std::sync::Arc;

use crate::{
    application::{auth_service::AuthService, blog_service::BlogService},
    domain::error::{PostError, UserError},
    infrastructure::jwt::JwtService,
    presentation::{self, dto::PaginactionQuery, middleware::AuthentificatedUser},
};
use actix_web::{
    HttpResponse, Responder, delete, get, post, put,
    web::{self},
};

use crate::presentation::dto;

#[post("/register")]
pub async fn register(
    auth_service: actix_web::web::Data<Arc<AuthService>>,
    register: web::Json<dto::RegisterRequest>,
    jwt_service: web::Data<Arc<JwtService>>,
) -> Result<impl Responder, UserError> {
    match auth_service
        .register_user(register.0, &jwt_service.into_inner())
        .await
    {
        Ok(logger_res) => Ok(HttpResponse::Ok().json(logger_res)),
        Err(e) => {
            tracing::error!("user register error: {}", e);
            Err(e)
        }
    }
}

#[post("/login")]
pub async fn login(
    auth_service: actix_web::web::Data<Arc<AuthService>>,
    jwt_service: actix_web::web::Data<Arc<JwtService>>,
    login: web::Json<dto::LoginRequest>,
) -> Result<impl Responder, UserError> {
    match auth_service
        .login_user(login.0, &jwt_service.into_inner())
        .await
    {
        Ok(logged_res) => Ok(HttpResponse::Ok().json(logged_res)),
        Err(e) => {
            tracing::error!("user login error: {}", e);
            return Err(e);
        }
    }
}

#[post("/api/posts")]
pub async fn create_post(
    user: AuthentificatedUser,
    post: web::Json<dto::CreatePostRequest>,
    blog_service: web::Data<Arc<BlogService>>,
) -> Result<impl Responder, PostError> {
    match blog_service.add_post(post.0, user.get_id()).await {
        Ok(post) => Ok(HttpResponse::Created().json(post)),
        Err(e) => {
            tracing::error!("post create error: {e}");
            Err(e)
        }
    }
}

#[get("/api/posts/{id}")]
pub async fn get_post(
    path: web::Path<u32>,
    blog_service: web::Data<Arc<BlogService>>,
) -> Result<impl Responder, PostError> {
    let post_id = path.into_inner();
    match blog_service.find_post(post_id).await {
        Ok(post) => Ok(web::Json(post)),
        Err(e) => {
            tracing::error!("post get error: {e}");
            Err(e)
        }
    }
}

#[put("/api/posts/{id}")]
pub async fn update_post(
    user: AuthentificatedUser,
    path: web::Path<u32>,
    data: web::Json<dto::UpdatePostRequest>,
    blog_service: web::Data<Arc<BlogService>>,
) -> Result<impl Responder, PostError> {
    let post_id = path.into_inner();
    blog_service
        .update_post(post_id, data.0, user.get_id())
        .await
        .map(|r| web::Json(r))
}

#[delete("/api/posts/{id}")]
pub async fn delete_post(
    user: AuthentificatedUser,
    path: web::Path<u32>,
    blog_service: web::Data<Arc<BlogService>>,
) -> Result<impl Responder, PostError> {
    let post_id = path.into_inner();
    blog_service
        .delete_post(post_id, user.get_id())
        .await
        .map(|_| HttpResponse::Ok())
}

#[get("/api/posts")]
pub async fn get_posts(
    pagination: web::Query<PaginactionQuery>,
    blog_service: web::Data<Arc<BlogService>>,
) -> Result<impl Responder, PostError> {
    let offset = pagination
        .offset
        .unwrap_or(presentation::DEFAULT_LIST_OFFSET);
    let limit = pagination.limit.unwrap_or(presentation::DEFAULT_LIST_LIMIT);
    let total_count = blog_service.post_total_count().await?;
    blog_service.post_list(offset, limit).await.map(|v| {
        web::Json(serde_json::json!({
            "posts": v,
            "limit": limit,
            "offset": offset,
            "total": total_count,
        }))
    })
}
