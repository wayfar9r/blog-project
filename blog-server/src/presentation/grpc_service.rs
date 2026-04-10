use std::sync::Arc;

use tonic::metadata::MetadataMap;

use crate::{
    application::{
        auth_service::{self},
        blog_service::{self},
    },
    infrastructure,
    presentation::{
        self,
        exchange::{
            AuthResponse, EmptyResponse, LoginRequest, PostCreateRequest, PostDeleteRequest,
            PostGetRequest, PostListRequest, PostListResponse, PostResponse, PostUpdateRequest,
            RegisterRequest, blog_service_server::BlogService,
        },
    },
};

pub struct BlogGrpcService {
    auth_service: Arc<auth_service::AuthService>,
    blog_service: Arc<blog_service::BlogService>,
    jwt_service: Arc<infrastructure::jwt::JwtService>,
}

impl BlogGrpcService {
    pub fn new(
        auth_service: Arc<auth_service::AuthService>,
        blog_service: Arc<blog_service::BlogService>,
        jwt_service: Arc<infrastructure::jwt::JwtService>,
    ) -> Self {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }
}

fn get_auth_token(metadata: &MetadataMap) -> Result<String, tonic::Status> {
    metadata
        .get("authorization")
        .ok_or(tonic::Status::unauthenticated(
            "missing authorization header",
        ))?
        .to_str()
        .map_err(|e| tonic::Status::unauthenticated(e.to_string()))?
        .strip_prefix("Bearer ")
        .ok_or(tonic::Status::unauthenticated("Bearer not found"))
        .map(|v| v.to_owned())
}

#[tonic::async_trait]
impl BlogService for BlogGrpcService {
    async fn register(
        &self,
        request: tonic::Request<RegisterRequest>,
    ) -> std::result::Result<tonic::Response<AuthResponse>, tonic::Status> {
        self.auth_service
            .register_user(request.into_inner().into(), &self.jwt_service)
            .await
            .map(|r| tonic::Response::new(r.into()))
            .map_err(|e| e.into())
    }

    async fn login(
        &self,
        request: tonic::Request<LoginRequest>,
    ) -> std::result::Result<tonic::Response<AuthResponse>, tonic::Status> {
        self.auth_service
            .login_user(request.into_inner().into(), &self.jwt_service)
            .await
            .map(|r| tonic::Response::new(r.into()))
            .map_err(|e| e.into())
    }
    /// Create post request
    async fn create_post(
        &self,
        request: tonic::Request<PostCreateRequest>,
    ) -> std::result::Result<tonic::Response<PostResponse>, tonic::Status> {
        let token = get_auth_token(request.metadata())?;
        let claims = self
            .jwt_service
            .verify_token(&token)
            .map_err(|e| tonic::Status::unauthenticated(e.to_string()))?;
        self.blog_service
            .add_post(request.into_inner().into(), claims.get_user_id())
            .await
            .map(|p| tonic::Response::new(p.into()))
            .map_err(|e| e.into())
    }
    /// Get post request
    async fn get_post(
        &self,
        request: tonic::Request<PostGetRequest>,
    ) -> std::result::Result<tonic::Response<PostResponse>, tonic::Status> {
        let post_id = request.into_inner().id;
        self.blog_service
            .find_post(post_id)
            .await
            .map(|r| tonic::Response::new(r.into()))
            .map_err(|e| e.into())
    }
    /// Update post request
    async fn update_post(
        &self,
        request: tonic::Request<PostUpdateRequest>,
    ) -> std::result::Result<tonic::Response<PostResponse>, tonic::Status> {
        let token = get_auth_token(request.metadata())?;
        let claims = self
            .jwt_service
            .verify_token(&token)
            .map_err(|e| tonic::Status::unauthenticated(e.to_string()))?;
        let (post_id, data) = request.into_inner().into();
        self.blog_service
            .update_post(post_id, data, claims.get_user_id())
            .await
            .map(|r| tonic::Response::new(r.into()))
            .map_err(|e| e.into())
    }

    /// Delete post request
    async fn delete_posts(
        &self,
        request: tonic::Request<PostDeleteRequest>,
    ) -> std::result::Result<tonic::Response<EmptyResponse>, tonic::Status> {
        let token = get_auth_token(request.metadata())?;
        let claims = self
            .jwt_service
            .verify_token(&token)
            .map_err(|e| tonic::Status::unauthenticated(e.to_string()))?;
        self.blog_service
            .delete_post(request.into_inner().id, claims.get_user_id())
            .await
            .map(|_| tonic::Response::new(EmptyResponse {}))
            .map_err(|e| e.into())
    }

    /// List posts
    async fn list_posts(
        &self,
        request: tonic::Request<PostListRequest>,
    ) -> std::result::Result<tonic::Response<PostListResponse>, tonic::Status> {
        let pagination = request.into_inner();
        let limit = match pagination.limit {
            Some(v) => {
                if v > 255 {
                    return Err(tonic::Status::invalid_argument("limit is <= 255"));
                }
                v
            }
            None => presentation::DEFAULT_LIST_LIMIT as u32,
        };
        let offset = pagination
            .offset
            .unwrap_or(presentation::DEFAULT_LIST_OFFSET);
        let total = self.blog_service.post_total_count().await?;
        self.blog_service
            .post_list(offset, limit as u8)
            .await
            .map(|list| {
                tonic::Response::new(PostListResponse {
                    posts: list.into_iter().map(|p| p.into()).collect(),
                    total: total,
                    limit: limit,
                    offset: offset,
                })
            })
            .map_err(|e| e.into())
    }
}
