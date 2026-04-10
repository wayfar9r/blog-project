use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::domain::post::Post;

#[derive(Deserialize, Debug, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "username min length is 1"))]
    pub username: String,
    #[validate(length(
        min = 8,
        max = 256,
        message = "password length must be between 8 and 256"
    ))]
    pub password: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct RegisterRequest {
    #[validate(length(
        min = 1,
        max = 256,
        message = "username length must be between 1 and 256"
    ))]
    pub username: String,
    #[validate(email(message = "invalid email"))]
    pub email: String,
    #[validate(length(
        min = 8,
        max = 256,
        message = "password length must be between 8 and 256"
    ))]
    pub password: String,
}

#[allow(unused)]
type PostId = u32;
type PostTitle = String;
type PostContent = String;
type _PostAuthor = u32;

#[derive(Deserialize, Debug, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 256, message = "length must be between 1 and 256"))]
    pub title: PostTitle,
    #[validate(length(min = 1, max = 65636, message = "length must be between 1 and 65636"))]
    pub content: PostContent,
}

#[derive(Deserialize, Debug, Validate)]
pub struct UpdatePostRequest {
    #[validate(length(min = 1, max = 256, message = "length must be between 1 and 256"))]
    pub title: Option<PostTitle>,
    #[validate(length(min = 1, max = 65636, message = "length must be between 1 and 65636"))]
    pub content: Option<PostContent>,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: u32,
    pub username: String,
    pub email: String,
}

#[derive(Serialize, Debug)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PaginactionQuery {
    #[validate(range(min = 0, max = 4_294_967_295u32))]
    pub offset: Option<u32>,
    #[validate(range(min = 0, max = 255u8))]
    pub limit: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct PostListResponse {
    posts: Vec<Post>,
    total: u32,
    limit: u8,
}
