use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Body, Method, Response, Url};
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisErr;

#[derive(Clone)]
pub struct Client {
    service_base_url: Url,
    client_id: String,
    secret: String,
    redirect_uri: String,
}

impl Client {
    pub fn new(client_id: String, secret: String, redirect_uri: String) -> Self {
        Client {
            service_base_url: "https://app.misoca.jp".parse().unwrap(),
            client_id,
            secret,
            redirect_uri,
        }
    }

    async fn call(&self, input: CallInput) -> MisocaResult<Response> {
        let mut url = self.service_base_url.clone();
        url.set_path(format!("{}", input.path).as_str());
        println!("call api: {}", url.to_string());
        let mut req = reqwest::Request::new(input.method, url);
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            HeaderValue::from_str("application/json").unwrap(),
        );
        *req.headers_mut() = headers;
        *req.body_mut() = input.body;

        let cli = reqwest::Client::new();
        let resp = cli.execute(req).await.map_err(|e| -> MisocaError {
            println!("error: {}", e.to_string());
            MisocaError::from(e)
        })?;

        // let for_debug = &resp;
        // let body = for_debug.text().await;
        // if let Ok(body) = body {
        //     println!("response: {}", body)
        // }

        Ok(resp)
    }

    pub async fn get_refresh_token(
        &self,
        input: misoca_get_refresh_token::Input,
    ) -> MisocaResult<misoca_get_refresh_token::Output> {
        #[derive(Debug, Serialize)]
        struct Input {
            pub client_id: String,
            pub client_secret: String,
            pub redirect_uri: String,
            pub grant_type: String,
            pub code: String,
        }

        let _input = Input {
            client_id: self.client_id.clone(),
            client_secret: self.secret.clone(),
            redirect_uri: self.redirect_uri.clone(),
            grant_type: "authorization_code".to_string(),
            code: input.code.clone(),
        };

        println!("json body: {}", serde_json::to_string(&_input).unwrap());

        self.call(CallInput {
            method: Method::POST,
            path: "/oauth2/token".to_string(),
            body: Some(
                serde_json::to_string(&_input)
                    .map_err(|e| MisocaError::Internal(e.to_string()))?
                    .into(),
            ),
        })
        .await?
        .error_for_status()?
        .json::<misoca_get_refresh_token::Output>()
        .await
        .map_err(MisocaError::from)
    }
}

pub mod misoca_get_refresh_token {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct Input {
        pub code: String,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Output {
        pub refresh_token: String,
    }
}

#[derive(Default)]
pub struct CallInput {
    pub method: Method,
    pub path: String,
    pub body: Option<Body>,
}

#[derive(ThisErr, Debug)]
pub enum MisocaError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub type MisocaResult<T> = Result<T, MisocaError>;

impl From<reqwest::Error> for MisocaError {
    fn from(v: reqwest::Error) -> Self {
        Self::Internal(v.to_string())
    }
}
