use reqwest::{Response, StatusCode};

use crate::{AuthResponse, Post, PostList, error::BlogClientError};

#[derive(Debug)]
pub struct HttpClient {
    api_url: String,
    client: reqwest::Client,
}

async fn map_not_ok_status(status: StatusCode, response: Response) -> BlogClientError {
    match status {
        StatusCode::NOT_FOUND => BlogClientError::NotFound,
        StatusCode::BAD_REQUEST => {
            let r = match response.text().await {
                Ok(r) => r,
                Err(e) => return BlogClientError::from(e),
            };
            BlogClientError::InvalidRequest(r)
        }
        status => BlogClientError::UnexpectedStatus {
            status: status.as_u16(),
            text: match response.text().await {
                Ok(t) => t,
                Err(e) => return BlogClientError::from(e),
            },
        },
    }
}

impl HttpClient {
    pub fn new(api_url: String) -> Self {
        let client = reqwest::Client::new();
        HttpClient { client, api_url }
    }

    fn auth_api_url(&self) -> String {
        format!("{}/api/auth/login", &self.api_url)
    }

    fn reg_api_url(&self) -> String {
        format!("{}/api/auth/register", &self.api_url)
    }

    fn create_api_url(&self) -> String {
        format!("{}/api/posts", &self.api_url)
    }

    fn post_put_api_url(&self, id: u32) -> String {
        format!("{}/api/posts/{}", &self.api_url, id)
    }

    fn get_post_api_url(&self, id: u32) -> String {
        format!("{}/api/posts/{}", self.api_url, id)
    }

    fn delete_post_api_url(&self, id: u32) -> String {
        format!("{}/api/posts/{}", self.api_url, id)
    }

    pub async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        match self
            .client
            .execute(
                self.client
                    .post(self.reg_api_url())
                    .json(&serde_json::json!({
                        "username": username,
                        "email": email,
                        "password": password,
                    }))
                    .build()?,
            )
            .await
        {
            Ok(r) => match r.status() {
                StatusCode::OK | StatusCode::CREATED => Ok(r.json::<AuthResponse>().await?),
                status => Err(map_not_ok_status(status, r).await),
            },
            Err(e) => Err(BlogClientError::HttpClient(e)),
        }
    }

    pub async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        match self
            .client
            .execute(
                self.client
                    .post(self.auth_api_url())
                    .json(&serde_json::json!({
                        "username": username,
                        "password": password
                    }))
                    .build()?,
            )
            .await
        {
            Ok(r) => match r.status() {
                StatusCode::OK => Ok(r.json::<AuthResponse>().await?),
                status => Err(map_not_ok_status(status, r).await),
            },
            Err(e) => Err(BlogClientError::HttpClient(e)),
        }
    }

    pub async fn create_post(
        &self,
        title: String,
        content: String,
        token: String,
    ) -> Result<Post, BlogClientError> {
        match self
            .client
            .execute(
                self.client
                    .post(self.create_api_url())
                    .bearer_auth(token)
                    .json(&serde_json::json!({
                        "title": title,
                        "content": content
                    }))
                    .build()?,
            )
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::OK | StatusCode::CREATED => Ok(response.json::<Post>().await?),
                status => Err(map_not_ok_status(status, response).await),
            },
            Err(e) => Err(BlogClientError::from(e)),
        }
    }

    pub async fn update_post(
        &self,
        id: u32,
        title: String,
        content: String,
        token: String,
    ) -> Result<Post, BlogClientError> {
        match self
            .client
            .execute(
                self.client
                    .put(self.post_put_api_url(id))
                    .bearer_auth(token)
                    .json(&serde_json::json!({
                        "title": title,
                        "content": content
                    }))
                    .build()?,
            )
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::OK => Ok(response.json::<Post>().await?),
                status => Err(map_not_ok_status(status, response).await),
            },
            Err(e) => Err(BlogClientError::ClientError(e.to_string())),
        }
    }

    pub async fn get_post(&self, id: u32) -> Result<Post, BlogClientError> {
        match self
            .client
            .execute(self.client.get(self.get_post_api_url(id)).build()?)
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::OK => Ok(response.json::<Post>().await?),
                status => Err(map_not_ok_status(status, response).await),
            },
            Err(e) => Err(BlogClientError::ClientError(e.to_string())),
        }
    }

    pub async fn delete_post(&self, id: u32, token: String) -> Result<(), BlogClientError> {
        match self
            .client
            .execute(
                self.client
                    .delete(self.delete_post_api_url(id))
                    .bearer_auth(token)
                    .build()?,
            )
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::OK => Ok(()),
                status => Err(map_not_ok_status(status, response).await),
            },
            Err(e) => Err(BlogClientError::ClientError(e.to_string())),
        }
    }

    pub async fn list_posts(&self, offset: u32, limit: u8) -> Result<PostList, BlogClientError> {
        match self
            .client
            .execute(
                self.client
                    .get(format!(
                        "{}/api/posts?offset={}&limit={}",
                        self.api_url, offset, limit
                    ))
                    .build()?,
            )
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::OK => Ok(response.json::<PostList>().await?),
                status => Err(map_not_ok_status(status, response).await),
            },
            Err(e) => Err(BlogClientError::from(e)),
        }
    }
}
