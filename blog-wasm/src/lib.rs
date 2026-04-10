use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::Storage;

#[wasm_bindgen]
pub struct BlogApp {
    remote_server: String,
    #[allow(unused)]
    token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: u32,
    username: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AuthentificatedUser {
    token: String,
    user: User,
}

#[derive(Debug, Deserialize, Serialize)]
struct Post {
    id: u32,
    author_id: u32,
    title: String,
    content: String,
    updated_at: String,
    created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PostList {
    offset: u32,
    total: u32,
    limit: u8,
    posts: Vec<Post>,
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new(remote_server: String) -> Self {
        BlogApp {
            remote_server,
            token: None,
        }
    }

    pub async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        match gloo_net::http::Request::post(&format!("{}/api/auth/register", self.remote_server))
            .json(&serde_json::json!({
                "username": username,
                "email": email,
                "password": password
            }))
            .map_err(|e| JsValue::from(format!("failed to build json body: {e}")))?
            .send()
            .await
        {
            Ok(r) => match r.status() {
                200 | 201 => {
                    let auth_user = r.json::<AuthentificatedUser>().await.map_err(|e| {
                        JsValue::from(format!("failed to parse auth info as json. {e}"))
                    })?;
                    self.save_token_to_storage(&auth_user.token)?;
                    self.save_user_info_to_storage(&auth_user.user)?;
                    Ok(serde_wasm_bindgen::to_value(&auth_user)?)
                }
                _ => Err(JsValue::from_str(&r.text().await.map_err(|e| {
                    JsValue::from(format!("failed to parse response text. {e}"))
                })?)),
            },
            Err(e) => Err(JsValue::from(e.to_string())),
        }
    }

    pub async fn login(&self, username: String, password: String) -> Result<JsValue, JsValue> {
        match gloo_net::http::Request::post(&format!("{}/api/auth/login", self.remote_server))
            .json(&serde_json::json!({
                "username": username,
                "password": password
            }))
            .map_err(|e| JsValue::from(format!("failed to build json body: {e}")))?
            .send()
            .await
        {
            Ok(r) => match r.status() {
                200 => {
                    let auth_user = r
                        .json::<AuthentificatedUser>()
                        .await
                        .map_err(|e| JsValue::from(format!("failed to parse post as json: {e}")))?;
                    self.save_token_to_storage(&auth_user.token)?;
                    self.save_user_info_to_storage(&auth_user.user)?;
                    Ok(serde_wasm_bindgen::to_value(&auth_user)?)
                }
                _ => Err(JsValue::from_str(&r.text().await.map_err(|e| {
                    JsValue::from(format!("failed to parse response text: {e}"))
                })?)),
            },
            Err(e) => Err(JsValue::from(e.to_string())),
        }
    }

    pub async fn load_posts(&self, offset: u32, limit: u8) -> Result<JsValue, JsValue> {
        match gloo_net::http::Request::get(&format!(
            "{}/api/posts?offset={}&limit={}",
            self.remote_server, offset, limit
        ))
        .send()
        .await
        {
            Ok(r) => match r.status() {
                200 => {
                    let post_list = r
                        .json::<PostList>()
                        .await
                        .map_err(|e| JsValue::from(e.to_string()))?;
                    Ok(serde_wasm_bindgen::to_value(&post_list)?)
                }
                _ => Err(JsValue::from_str(
                    &r.text().await.map_err(|e| JsValue::from(e.to_string()))?,
                )),
            },
            Err(e) => Err(JsValue::from(e.to_string())),
        }
    }

    pub async fn create_post(&self, title: String, content: String) -> Result<JsValue, JsValue> {
        match gloo_net::http::Request::post(&format!("{}/api/posts", self.remote_server))
            .header(
                "authorization",
                &format!(
                    "Bearer {}",
                    self.get_token_from_storage()?
                        .ok_or(JsValue::from_str("Токен не найден"))?
                ),
            )
            .json(&serde_json::json!({
                "title": title,
                "content": content,
            }))
            .map_err(|e| JsValue::from(format!("failed to build json: {e}")))?
            .send()
            .await
        {
            Ok(r) => match r.status() {
                200 | 201 => {
                    let post = r
                        .json::<Post>()
                        .await
                        .map_err(|e| JsValue::from(e.to_string()))?;
                    Ok(serde_wasm_bindgen::to_value(&post)?)
                }
                _ => Err(JsValue::from_str(
                    &r.text().await.map_err(|e| JsValue::from(e.to_string()))?,
                )),
            },
            Err(e) => Err(JsValue::from(e.to_string())),
        }
    }

    pub async fn update_post(
        &self,
        id: u32,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<JsValue, JsValue> {
        match gloo_net::http::Request::put(&format!("{}/api/posts/{}", self.remote_server, id))
            .header(
                "authorization",
                &format!(
                    "Bearer {}",
                    self.get_token_from_storage()?
                        .ok_or(JsValue::from_str("Токен не найден"))?
                ),
            )
            .json(&serde_json::json!({
                "title": title,
                "content": content,
            }))
            .map_err(|_e| JsValue::from("Ошибка при формировании json"))?
            .send()
            .await
        {
            Ok(r) => match r.status() {
                200 => {
                    let post = r
                        .json::<Post>()
                        .await
                        .map_err(|e| JsValue::from(e.to_string()))?;
                    Ok(serde_wasm_bindgen::to_value(&post)?)
                }
                _ => Err(JsValue::from_str(
                    &r.text().await.map_err(|e| JsValue::from(e.to_string()))?,
                )),
            },
            Err(e) => Err(JsValue::from(e.to_string())),
        }
    }

    pub async fn delete_post(&self, id: u32) -> Result<JsValue, JsValue> {
        match gloo_net::http::Request::delete(&format!("{}/api/posts/{id}", self.remote_server))
            .header(
                "authorization",
                &format!(
                    "Bearer {}",
                    self.get_token_from_storage()?
                        .ok_or(JsValue::from_str("Токен не найден"))?
                ),
            )
            .send()
            .await
        {
            Ok(r) => match r.status() {
                200 => Ok(JsValue::null()),
                _ => Err(JsValue::from_str(
                    &r.text().await.map_err(|e| JsValue::from(e.to_string()))?,
                )),
            },
            Err(e) => Err(JsValue::from(e.to_string())),
        }
    }

    pub fn is_authentificated(&self) -> JsValue {
        JsValue::from_bool(self.get_token_from_storage().is_ok_and(|v| v.is_some()))
    }

    fn get_token_from_storage(&self) -> Result<Option<String>, JsValue> {
        let storage = self.get_storage()?;
        storage.get_item("blog_token")
    }

    fn get_storage(&self) -> Result<Storage, JsValue> {
        web_sys::window()
            .ok_or(JsValue::from_str("Не удалось получить доступ к window"))?
            .local_storage()?
            .ok_or(JsValue::from_str(
                "Не удалось получить доступ к local storage",
            ))
    }

    fn save_token_to_storage(&self, token: &str) -> Result<(), JsValue> {
        let storage = self.get_storage()?;
        storage.set_item("blog_token", token)?;
        Ok(())
    }

    fn save_user_info_to_storage(&self, user: &User) -> Result<(), JsValue> {
        let storage = self.get_storage()?;
        storage.set_item("user", &serde_json::json!(&user).to_string())?;
        Ok(())
    }

    pub fn get_user_info_from_storage(&self) -> Result<JsValue, JsValue> {
        let storage = self.get_storage()?;
        match storage.get_item("user") {
            Ok(r) => js_sys::JSON::parse(&r.ok_or(JsValue::from_str("failed to get user info"))?),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
