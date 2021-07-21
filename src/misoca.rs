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

    async fn call(&self, input: CallInput, token: String) -> MisocaResult<Response> {
        let mut url = self.service_base_url.clone();
        url.set_path(format!("{}", input.path).as_str());
        for q in input.query {
            url.set_query(Some(q.as_str()))
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
        let resp = cli.execute(req).await.map_err(|e| -> MisocaError {
            println!("error: {}", e.to_string());
            MisocaError::from(e)
        })?;
        Ok(resp)
    }

    pub async fn get_tokens(&self, input: get_tokens::Input) -> MisocaResult<get_tokens::Output> {
        #[derive(Debug, Serialize)]
        struct Body {
            pub client_id: String,
            pub client_secret: String,
            pub redirect_uri: String,
            pub grant_type: String,
            pub code: String,
        }

        let body = Body {
            client_id: self.client_id.clone(),
            client_secret: self.secret.clone(),
            redirect_uri: self.redirect_uri.clone(),
            grant_type: "authorization_code".to_string(),
            code: input.code.clone(),
        };

        println!("json body: {}", serde_json::to_string(&body).unwrap());

        let query = vec![];

        self.call(
            CallInput {
                method: Method::POST,
                path: "/oauth2/token".to_string(),
                body: Some(
                    serde_json::to_string(&body)
                        .map_err(|e| MisocaError::Internal(e.to_string()))?
                        .into(),
                ),
                query,
            },
            "".to_string(),
        )
        .await?
        .error_for_status()?
        .json::<get_tokens::Output>()
        .await
        .map_err(MisocaError::from)
    }

    pub async fn refresh_tokens(
        &self,
        input: refresh_tokens::Input,
    ) -> MisocaResult<refresh_tokens::Output> {
        #[derive(Debug, Serialize)]
        struct Body {
            pub client_id: String,
            pub client_secret: String,
            pub redirect_uri: String,
            pub grant_type: String,
            pub refresh_token: String,
        }

        let body = Body {
            client_id: self.client_id.clone(),
            client_secret: self.secret.clone(),
            redirect_uri: self.redirect_uri.clone(),
            grant_type: "refresh_token".to_string(),
            refresh_token: input.refresh_token,
        };

        println!("json body: {}", serde_json::to_string(&body).unwrap());

        let query = vec![];

        self.call(
            CallInput {
                method: Method::POST,
                path: "/oauth2/token".to_string(),
                body: Some(
                    serde_json::to_string(&body)
                        .map_err(|e| MisocaError::Internal(e.to_string()))?
                        .into(),
                ),
                query,
            },
            "".to_string(),
        )
        .await?
        .error_for_status()?
        .json::<refresh_tokens::Output>()
        .await
        .map_err(MisocaError::from)
    }

    pub async fn get_invoices(
        &self,
        input: get_invoices::Input,
    ) -> MisocaResult<get_invoices::Output> {
        #[derive(Debug, Serialize)]
        struct Body {}

        let body = Body {};

        println!("json body: {}", serde_json::to_string(&body).unwrap());

        let query = vec![
            format!("condition={}", input.supplier_name),
            format!("page={}", input.page),
            format!("per_page={}", input.per_page),
        ];

        self.call(
            CallInput {
                method: Method::GET,
                path: "/api/v3/invoices".to_string(),
                body: Some(
                    serde_json::to_string(&body)
                        .map_err(|e| MisocaError::Internal(e.to_string()))?
                        .into(),
                ),
                query,
            },
            input.access_token,
        )
        .await?
        .error_for_status()?
        .json::<get_invoices::Output>()
        .await
        .map_err(MisocaError::from)
    }
}

pub mod get_tokens {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct Input {
        pub code: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Output {
        pub access_token: String,
        pub refresh_token: String,
    }
}

pub mod refresh_tokens {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct Input {
        pub refresh_token: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Output {
        pub access_token: String,
        pub refresh_token: String,
    }
}

pub mod get_invoices {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct Input {
        pub access_token: String,
        pub supplier_name: String,
        pub page: i32,
        pub per_page: i32,
    }

    pub type Output = Vec<Invoice>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Option<i32>,
    pub issue_date: Option<String>,
    pub payment_due_on: Option<String>,
    pub invoice_number: Option<String>,
    pub payment_status: Option<i32>,
    pub invoice_status: Option<i32>,
    pub recipient_name: String,
    pub subject: Option<String>,
    pub body: Option<InvoiceBody>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceBody {
    pub total_amount: Option<String>,
    pub tax: Option<String>,
}

#[derive(Default)]
pub struct CallInput {
    pub method: Method,
    pub path: String,
    pub body: Option<Body>,
    pub query: Vec<String>,
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
