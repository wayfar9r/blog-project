pub mod post_repository;
pub mod user_repository;

use crate::presentation::dto::CreatePostRequest;

impl From<(CreatePostRequest, u32)> for CreatePostData {
    fn from((post_request, author): (CreatePostRequest, u32)) -> Self {
        CreatePostData {
            title: post_request.title,
            content: post_request.content,
            author,
        }
    }
}

pub struct CreatePostData {
    title: String,
    content: String,
    author: u32,
}
