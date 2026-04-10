use crate::{
    domain::{
        self,
        error::{PostError, UserError},
    },
    presentation::{
        dto,
        exchange::{
            AuthResponse, LoginRequest, PostCreateRequest, PostResponse, PostUpdateRequest,
            RegisterRequest, User,
        },
    },
};

impl From<RegisterRequest> for dto::RegisterRequest {
    fn from(value: RegisterRequest) -> Self {
        dto::RegisterRequest {
            username: value.username,
            email: value.email,
            password: value.password,
        }
    }
}

impl From<dto::AuthResponse> for AuthResponse {
    fn from(value: dto::AuthResponse) -> Self {
        AuthResponse {
            token: value.token,
            user: Some(User {
                id: value.user.id,
                username: value.user.username,
                email: value.user.email,
            }),
        }
    }
}

impl From<LoginRequest> for dto::LoginRequest {
    fn from(value: LoginRequest) -> Self {
        dto::LoginRequest {
            username: value.username,
            password: value.password,
        }
    }
}

impl From<PostError> for tonic::Status {
    fn from(value: PostError) -> Self {
        match value {
            PostError::Forbidden(e) => tonic::Status::permission_denied(e),
            PostError::NotFound(e) => tonic::Status::not_found(e),
            PostError::UnexpectedFailure(e) => tonic::Status::internal(e),
            PostError::Validation(e) => tonic::Status::invalid_argument(e.to_string()),
        }
    }
}

impl From<UserError> for tonic::Status {
    fn from(value: UserError) -> Self {
        match value {
            UserError::InvalidCredentials(e) => tonic::Status::permission_denied(e),
            UserError::NotFound(e) => tonic::Status::not_found(e),
            UserError::UnexpectedFailure(e) => tonic::Status::internal(e),
            UserError::UserAlreadyExists(e) => tonic::Status::already_exists(e),
            UserError::Validation(e) => tonic::Status::invalid_argument(e.to_string()),
        }
    }
}

impl From<PostCreateRequest> for dto::CreatePostRequest {
    fn from(value: PostCreateRequest) -> Self {
        dto::CreatePostRequest {
            title: value.title,
            content: value.content,
        }
    }
}

impl From<domain::post::Post> for PostResponse {
    fn from(value: domain::post::Post) -> Self {
        PostResponse {
            id: value.id as u32,
            author_id: value.author_id as u32,
            title: value.title,
            content: value.content,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

impl From<PostUpdateRequest> for (u32, dto::UpdatePostRequest) {
    fn from(value: PostUpdateRequest) -> Self {
        (
            value.id,
            dto::UpdatePostRequest {
                title: value.title,
                content: value.content,
            },
        )
    }
}
