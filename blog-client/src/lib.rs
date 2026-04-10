//! Blog client
//!
//! Программа предлагает единый интерфейс для
//! общения с веб сервером и grpc сервисом
//!

#[warn(missing_docs)]

use serde::Deserialize;

use crate::{
    error::BlogClientError,
    exchange::{PostListResponse, PostResponse},
    grpc_client::GrpcClient,
    http_client::HttpClient,
};

pub mod error;
mod grpc_client;
mod http_client;

mod exchange {
    tonic::include_proto!("proto.blog");
}

///Транспорт по которому
/// будет идти общение
#[derive(Debug)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

///Публичная информация о пользователе
/// хранящаяся на клиенте
#[derive(Debug, Deserialize)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub email: String,
}

/// Публичная информация о авторизации
/// хранящаяся на клиенте
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: Option<User>,
}

/// Информация о посте для сериализации
/// и транспорта
#[derive(Debug, Deserialize)]
pub struct Post {
    pub id: u32,
    pub author_id: u32,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

///Структура списка постов для
/// сериализации и передачи
#[derive(Debug, Deserialize)]
pub struct PostList {
    pub posts: Vec<Post>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

impl From<PostListResponse> for PostList {
    fn from(value: PostListResponse) -> Self {
        PostList {
            posts: value.posts.into_iter().map(|v| v.into()).collect(),
            total: value.total,
            limit: value.limit,
            offset: value.offset,
        }
    }
}

impl From<PostResponse> for Post {
    fn from(value: PostResponse) -> Self {
        Post {
            id: value.id,
            author_id: value.author_id,
            title: value.title,
            content: value.content,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<exchange::AuthResponse> for AuthResponse {
    fn from(value: exchange::AuthResponse) -> Self {
        let user = value.user.map(|u| User {
            id: u.id,
            username: u.username,
            email: u.email,
        });
        AuthResponse {
            user,
            token: value.token,
        }
    }
}

/// Клиент для взаимодействия
/// с веб сервером
#[derive(Debug)]
pub struct BlogClient {
    transport: Transport,
    http_client: Option<HttpClient>,
    grpc_client: Option<GrpcClient>,
    token: Option<String>,
}

fn modify_token(token: String) -> String {
    format!("Bearer {token}")
}

impl BlogClient {
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        let (http_client, grpc_client) = match &transport {
            Transport::Http(url) => (Some(HttpClient::new(url.clone())), None),
            Transport::Grpc(url) => (None, Some(GrpcClient::new(url.clone()).await?)),
        };
        Ok(Self {
            transport,
            http_client: http_client,
            grpc_client: grpc_client,
            token: None,
        })
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        match self.transport {
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .unwrap()
                    .register(username, email, password)
                    .await
            }
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .unwrap()
                    .register(username, email, password)
                    .await
            }
        }
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        match self.transport {
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .unwrap()
                    .login(username, password)
                    .await
            }
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .unwrap()
                    .login(username, password)
                    .await
            }
        }
    }

    pub async fn create_post(
        &mut self,
        title: String,
        content: String,
        token: String,
    ) -> Result<Post, BlogClientError> {
        match self.transport {
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .unwrap()
                    .create_post(title, content, modify_token(token))
                    .await
            }
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .unwrap()
                    .create_post(title, content, token)
                    .await
            }
        }
    }

    pub async fn get_post(&mut self, id: u32) -> Result<Post, BlogClientError> {
        match self.transport {
            Transport::Grpc(_) => self.grpc_client.as_mut().unwrap().get_post(id).await,
            Transport::Http(_) => self.http_client.as_ref().unwrap().get_post(id).await,
        }
    }

    pub async fn update_post(
        &mut self,
        id: u32,
        title: String,
        content: String,
        token: String,
    ) -> Result<Post, BlogClientError> {
        match self.transport {
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .unwrap()
                    .update_post(id, Some(title), Some(content), modify_token(token))
                    .await
            }
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .unwrap()
                    .update_post(id, title, content, token)
                    .await
            }
        }
    }

    pub async fn delete_post(&mut self, id: u32, token: String) -> Result<(), BlogClientError> {
        match self.transport {
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .unwrap()
                    .delete_post(id, modify_token(token))
                    .await
            }
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .unwrap()
                    .delete_post(id, token)
                    .await
            }
        }
    }

    pub async fn list_posts(
        &mut self,
        limit: u8,
        offset: u32,
    ) -> Result<PostList, BlogClientError> {
        match self.transport {
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .unwrap()
                    .list_posts(offset, limit)
                    .await
            }
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .unwrap()
                    .list_posts(offset, limit)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_client() {
        let mut blog_client = BlogClient::new(Transport::Http("http://127.0.0.1:3000".into()))
            .await
            .unwrap();
        blog_client
            .login("username".into(), "pass1".into())
            .await
            .unwrap();
    }
}
