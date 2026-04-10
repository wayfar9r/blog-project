use crate::error::BlogClientError;
use crate::exchange::{
    LoginRequest, PostCreateRequest, PostDeleteRequest, PostGetRequest, PostListRequest,
    PostUpdateRequest, RegisterRequest,
};
use crate::{AuthResponse, Post, PostList, exchange};
use exchange::blog_service_client::BlogServiceClient;
use tonic::IntoRequest;
use tonic::metadata::{Ascii, MetadataMap, MetadataValue};

#[derive(Debug)]
pub struct GrpcClient {
    client: BlogServiceClient<tonic::transport::Channel>,
}

fn inject_auth(metadata: &mut MetadataMap, token: String) -> Result<(), BlogClientError> {
    metadata.insert(
        "authorization",
        token
            .parse::<MetadataValue<Ascii>>()
            .map_err(|e| BlogClientError::ClientError(e.to_string()))?,
    );
    Ok(())
}

impl GrpcClient {
    pub async fn new(url: String) -> Result<Self, BlogClientError> {
        let client = BlogServiceClient::connect(url).await?;
        Ok(GrpcClient { client })
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .login(LoginRequest {
                username: username,
                password: password,
            })
            .await?;
        Ok(response.into_inner().into())
    }

    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .register(RegisterRequest {
                username,
                email,
                password,
            })
            .await?;
        Ok(response.into_inner().into())
    }

    pub async fn create_post(
        &mut self,
        title: String,
        content: String,
        token: String,
    ) -> Result<Post, BlogClientError> {
        let mut request = tonic::Request::new(PostCreateRequest { title, content });
        inject_auth(request.metadata_mut(), token)?;
        let response = self.client.create_post(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn update_post(
        &mut self,
        id: u32,
        title: Option<String>,
        content: Option<String>,
        token: String,
    ) -> Result<Post, BlogClientError> {
        let mut request = PostUpdateRequest { id, title, content }.into_request();
        inject_auth(request.metadata_mut(), token)?;
        let response = self.client.update_post(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn get_post(&mut self, id: u32) -> Result<Post, BlogClientError> {
        let response = self.client.get_post(PostGetRequest { id }).await?;
        Ok(response.into_inner().into())
    }

    pub async fn delete_post(&mut self, id: u32, token: String) -> Result<(), BlogClientError> {
        let mut request = PostDeleteRequest { id }.into_request();
        inject_auth(request.metadata_mut(), token)?;
        self.client.delete_posts(request).await?;
        Ok(())
    }

    pub async fn list_posts(
        &mut self,
        offset: u32,
        limit: u8,
    ) -> Result<PostList, BlogClientError> {
        let request = PostListRequest {
            offset: Some(offset),
            limit: Some(limit as u32),
        }
        .into_request();
        let response = self.client.list_posts(request).await?;
        Ok(response.into_inner().into())
    }
}
