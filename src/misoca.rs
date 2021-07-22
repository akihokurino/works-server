use crate::domain;
use crate::domain::YMD;
use crate::util;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Body, Method, Response, Url};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
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
        supplier: &domain::supplier::Supplier,
    ) -> MisocaResult<Vec<domain::invoice::Invoice>> {
        #[derive(Debug, Serialize)]
        struct Body {}

        let body = Body {};

        println!("json body: {}", serde_json::to_string(&body).unwrap());

        let query = vec![
            ("page".to_string(), input.page.to_string()),
            ("per_page".to_string(), input.per_page.to_string()),
            ("condition".to_string(), supplier.name.to_string()),
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
        .map(|item| {
            item.into_iter()
                .map(|v| v.to_domain(supplier).unwrap())
                .collect::<Vec<_>>()
        })
    }

    pub async fn get_pdf(&self, input: get_pdf::Input) -> MisocaResult<get_pdf::Output> {
        let client = reqwest::Client::new();
        let mut url = self.service_base_url.clone();
        url.set_path(format!("/api/v3/invoice/{}/pdf", input.invoice_id).as_str());
        let resp: Response = client
            .get(url)
            .header("Authorization", format!("bearer {}", input.access_token))
            .send()
            .await
            .map_err(MisocaError::from)?;
        let content = resp.text().await.map_err(MisocaError::from)?;
        Ok(content)
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
    pub recipient_name: Option<String>,
    pub subject: Option<String>,
    pub body: Option<InvoiceBody>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceBody {
    pub total_amount: Option<String>,
    pub tax: Option<String>,
}

impl Invoice {
    fn to_domain(
        &self,
        supplier: &domain::supplier::Supplier,
    ) -> MisocaResult<domain::invoice::Invoice> {
        let body = self.body.as_ref().unwrap();

        let total_amount: f64 = body
            .total_amount
            .clone()
            .unwrap_or("0".to_string())
            .parse()
            .unwrap();
        let tax: f64 = body.tax.clone().unwrap_or("0".to_string()).parse().unwrap();

        let issue_ymd =
            YMD::from_str(self.issue_date.clone().unwrap_or("".to_string()).as_str())
                .map_err(|_e| MisocaError::Internal("cannot parse issue_date".to_string()))?;
        let payment_due_on_ymd = YMD::from_str(
            self.payment_due_on
                .clone()
                .unwrap_or("".to_string())
                .as_str(),
        )
        .map_err(|_e| MisocaError::Internal("cannot parse payment_due_on".to_string()))?;

        let created_at =
            chrono::DateTime::parse_from_rfc3339(self.created_at.clone().unwrap().as_str())
                .map_err(|_e| MisocaError::Internal("cannot parse created_at".to_string()))?;
        let updated_at =
            chrono::DateTime::parse_from_rfc3339(self.updated_at.clone().unwrap().as_str())
                .map_err(|_e| MisocaError::Internal("cannot parse updated_at".to_string()))?;

        Ok(domain::invoice::Invoice {
            id: String::from(self.id.unwrap().to_string()),
            supplier_id: supplier.id.clone(),
            issue_ymd,
            payment_due_on_ymd,
            invoice_number: self.invoice_number.clone().unwrap_or("".to_string()),
            payment_status: domain::invoice::PaymentStatus::from(
                self.payment_status.clone().unwrap_or(0),
            ),
            invoice_status: domain::invoice::InvoiceStatus::from(
                self.invoice_status.clone().unwrap_or(0),
            ),
            recipient_name: self.recipient_name.clone().unwrap_or("".to_string()),
            subject: self.subject.clone().unwrap_or("".to_string()),
            total_amount: util::f64_to_i32(total_amount),
            tax: util::f64_to_i32(tax),
            pdf_path: None,
            created_at: created_at.naive_utc(),
            updated_at: updated_at.naive_utc(),
        })
    }
}

pub mod get_pdf {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct Input {
        pub access_token: String,
        pub invoice_id: String,
    }

    pub type Output = String;
}

#[derive(Default)]
pub struct CallInput {
    pub method: Method,
    pub path: String,
    pub body: Option<Body>,
    pub query: Vec<(String, String)>,
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
