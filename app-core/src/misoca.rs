pub mod contact;
pub mod invoice;
pub mod tokens;

use crate::{CoreError, CoreResult};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Body, Method, Response, Url};

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

    async fn call(&self, input: CallInput, token: String) -> CoreResult<Response> {
        let mut url = self.service_base_url.clone();
        url.set_path(format!("{}", input.path).as_str());
        for q in input.query {
            url.query_pairs_mut()
                .append_pair(q.0.as_str(), q.1.as_str());
        }
        println!("call api: {}", url.to_string());

        let mut req = reqwest::Request::new(input.method, url);

        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            HeaderValue::from_str("application/json").unwrap(),
        );
        if !token.is_empty() {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("bearer {}", token)).unwrap(),
            );
        }
        *req.headers_mut() = headers;

        *req.body_mut() = input.body;

        let cli = reqwest::Client::new();
        let resp = cli.execute(req).await.map_err(|e| -> CoreError {
            println!("error: {}", e.to_string());
            CoreError::from(e)
        })?;

        Ok(resp)
    }
}

#[derive(Default)]
pub struct CallInput {
    pub method: Method,
    pub path: String,
    pub body: Option<Body>,
    pub query: Vec<(String, String)>,
}
