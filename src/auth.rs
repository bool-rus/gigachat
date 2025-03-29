use chrono::{DateTime, Utc};
use tonic::service::Interceptor;
use tonic::Request;
use uuid::Uuid;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::Duration;
use chrono::serde::ts_milliseconds;
use thiserror::Error;

static AUTH_URL: &str = "https://ngw.devices.sberbank.ru:9443/api/v2/oauth";

/// Параметр scope, который передается в сервис авторизации 
#[derive(Debug, Clone, Copy)]
pub enum Scope {
    Pers,
    B2b,
    Corp,
}

impl AsRef<str> for Scope {
    fn as_ref(&self) -> &str {
        match self {
            Scope::Pers => "GIGACHAT_API_PERS",
            Scope::B2b => "GIGACHAT_API_B2B",
            Scope::Corp => "GIGACHAT_API_CORP",
        }
    }
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AuthResponse {
    access_token: String,
    #[serde(with = "ts_milliseconds")]
    expires_at: DateTime<Utc>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Transport error")]
    Reqwest(#[from]reqwest::Error),
    #[error("Parse auth response error")]
    Parse(#[from]serde_json::Error),
    #[error("Auth response")]
    AuthResponse(String),
}


async fn auth(token: &str, scope: Scope) -> Result<AuthResponse, Error> {
    let client = reqwest::Client::builder()
        .use_native_tls()
        .build()?;
    let mut params = HashMap::new();
    params.insert("scope", scope.as_ref());
    let response = client
        .post(AUTH_URL)
        .header("Accept", "application/json")
        .header("RqUID", Uuid::new_v4().to_string())
        .header("Authorization", format!("Basic {token}"))
        .form(&params)
        .send()
        .await?;
    if !response.status().is_success() {
        let text = response.text().await?;
        return Err(Error::AuthResponse(text));
    }
    let response: AuthResponse = serde_json::from_str(response.text().await?.as_str())?;

    Ok(response)
}

/// Служба авторизации. Представляет собой обновляемый токен.
#[derive(Debug, Clone)]
pub struct TokenInterceptor {
    token: Arc<RwLock<String>>,
}

impl TokenInterceptor {
    pub async fn new(token: impl ToString, scope: Scope) -> Result<Self, Error> {
        let auth_token = token.to_string();
        let AuthResponse { access_token, expires_at } = auth(&auth_token, scope).await?;
        let token = Arc::new(RwLock::new(access_token));
        let updatable = Arc::downgrade(&token);
        tokio::spawn(async move {
            let sleep_duration = expires_at - Utc::now();
            tokio::time::sleep(sleep_duration.to_std().unwrap()).await;
            while let Some(updatable) = updatable.upgrade() {
                let AuthResponse { access_token, expires_at } = match auth(&auth_token, scope).await {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("update token: {e}");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    },
                };
                *updatable.write().unwrap() = access_token;
                log::info!("access token updated");
                let sleep_duration = expires_at - Utc::now();
                tokio::time::sleep(sleep_duration.to_std().unwrap_or(Duration::from_secs(5))).await;
            }
        });
        Ok(Self { token })
    }
    pub fn get_token(&self) -> String {
        self.token.read().unwrap().clone()
    }
}

impl Interceptor for TokenInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, tonic::Status> {
        req.metadata_mut().append(
            "authorization",
            format!("Bearer {}", self.token.read().unwrap()).parse().unwrap(),
        );
        Ok(req)
    }
}


