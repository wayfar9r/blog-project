use crate::{
    data::post_repository::PostRepository,
    domain::{
        error::{PostError, ValidationError},
        post::Post,
    },
    presentation::dto,
};

use crate::data::post_repository::Field;

use validator::Validate;

pub struct BlogService {
    post_repository: PostRepository,
}

impl BlogService {
    pub fn new(post_repository: PostRepository) -> Self {
        BlogService { post_repository }
    }

    pub async fn add_post(
        &self,
        data: dto::CreatePostRequest,
        user: u32,
    ) -> Result<Post, PostError> {
        if let Err(e) = data.validate() {
            log::error!("create post error: {e}");
            return Err(PostError::Validation(ValidationError {
                error: "data is not valid".into(),
                details: e.to_string(),
            }));
        }
        self.post_repository
            .create((data, user).into())
            .await
            .map_err(|e| PostError::UnexpectedFailure(e.to_string()))
    }

    pub async fn find_post(&self, id: u32) -> Result<Post, PostError> {
        self.post_repository
            .find(id)
            .await
            .map_err(|err| match err {
                sqlx::Error::RowNotFound => {
                    PostError::NotFound(format!("post with id {} not found", id))
                }
                e => PostError::UnexpectedFailure(e.to_string()),
            })
    }

    pub async fn update_post(
        &self,
        post_id: u32,
        data: dto::UpdatePostRequest,
        user: u32,
    ) -> Result<Post, PostError> {
        if let Err(e) = data.validate() {
            return Err(PostError::Validation(ValidationError {
                error: "post is not valid".into(),
                details: e.to_string(),
            }));
        }
        let post = self.find_post(post_id).await?;
        if post.author_id as u32 != user {
            return Err(PostError::Forbidden(format!(
                "access denied to update post {} for user {}",
                post_id, user,
            )));
        };
        let fields: Vec<Field> = data.into();
        self.post_repository
            .update(post_id, fields)
            .await
            .map_err(|e| PostError::UnexpectedFailure(e.to_string()))
    }

    pub async fn delete_post(&self, post_id: u32, user: u32) -> Result<(), PostError> {
        let post = self.find_post(post_id).await?;
        if post.author_id as u32 != user {
            return Err(PostError::Forbidden(format!(
                "access denied to update post {} for user {}",
                post_id, user,
            )));
        }
        self.post_repository
            .delete(post_id)
            .await
            .map_err(|e| PostError::UnexpectedFailure(e.to_string()))
    }

    pub async fn post_list(&self, offset: u32, limit: u8) -> Result<Vec<Post>, PostError> {
        let order = crate::data::post_repository::Order::Asc;
        self.post_repository
            .list(offset, limit, order)
            .await
            .map_err(|e| PostError::UnexpectedFailure(e.to_string()))
    }

    pub async fn post_total_count(&self) -> Result<u32, PostError> {
        self.post_repository
            .count()
            .await
            .map_err(|e| PostError::UnexpectedFailure(e.to_string()))
    }
}
