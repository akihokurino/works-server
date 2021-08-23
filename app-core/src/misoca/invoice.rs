use crate::domain;
use crate::domain::YMD;
use crate::misoca::{CallInput, Client};
use crate::util;
use crate::{CoreError, CoreResult};
use actix_web::web::Bytes;
use reqwest::{Method, Response};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

impl Client {
    pub async fn get_invoices(
        &self,
        input: get_invoices::Input,
    ) -> CoreResult<Vec<domain::invoice::Invoice>> {
        #[derive(Debug, Serialize)]
        struct Body {}

        let body = Body {};

        println!("json body: {}", serde_json::to_string(&body).unwrap());

        let query = vec![
            ("page".to_string(), input.page.to_string()),
            ("per_page".to_string(), input.per_page.to_string()),
            (
                "contact_group_id".to_string(),
                input.contact_group_id.to_string(),
            ),
        ];

        self.call(
            CallInput {
                method: Method::GET,
                path: "/api/v3/invoices".to_string(),
                body: Some(
                    serde_json::to_string(&body)
                        .map_err(|e| CoreError::Internal(e.to_string()))?
                        .into(),
                ),
                query,
            },
            input.access_token.clone(),
        )
        .await?
        .error_for_status()?
        .json::<get_invoices::Output>()
        .await
        .map_err(CoreError::from)
        .map(|item| {
            item.into_iter()
                .map(|v| v.to_domain(input.supplier_id.clone()).unwrap())
                .collect::<Vec<_>>()
        })
    }

    pub async fn get_pdf(&self, input: get_pdf::Input) -> CoreResult<get_pdf::Output> {
        let client = reqwest::Client::new();
        let mut url = self.service_base_url.clone();
        url.set_path(format!("/api/v3/invoice/{}/pdf", input.invoice_id).as_str());
        let resp: Response = client
            .get(url)
            .header("Authorization", format!("bearer {}", input.access_token))
            .send()
            .await
            .map_err(CoreError::from)?;
        let bytes = resp.bytes().await.map_err(CoreError::from)?;
        Ok(bytes)
    }

    pub async fn create_invoice(
        &self,
        input: create_invoice::Input,
    ) -> CoreResult<domain::invoice::Invoice> {
        #[derive(Debug, Serialize)]
        struct ItemBody {
            pub name: String,
            pub quantity: i32,
            pub unit_price: i32,
            pub tax_type: String,
            pub excluding_withholding_tax: bool,
        }

        #[derive(Debug, Serialize)]
        struct Sender {
            pub sender_name1: String,
            pub sender_zip_code: String,
            pub sender_address1: String,
            pub sender_tel: String,
            pub sender_email: String,
            pub bank_accounts: Vec<Bank>,
        }

        #[derive(Debug, Serialize)]
        struct Bank {
            pub detail: String,
        }

        #[derive(Debug, Serialize)]
        struct Body {
            pub issue_date: String,
            pub subject: String,
            pub payment_due_on: String,
            pub contact_id: i32,
            pub items: Vec<ItemBody>,
            pub body: Sender,
        }

        let banks = if let Some(bank) = input.bank.clone() {
            vec![Bank {
                detail: format!(
                    "{}（{}）（{}）口座番号：{}",
                    bank.name,
                    bank.code,
                    bank.account_type.to_string(),
                    bank.account_number
                ),
            }]
        } else {
            vec![]
        };

        let sender = if let Some(sender) = input.sender.clone() {
            Sender {
                sender_name1: sender.name,
                sender_zip_code: sender.postal_code,
                sender_address1: sender.address,
                sender_tel: sender.tel,
                sender_email: sender.email,
                bank_accounts: banks,
            }
        } else {
            Sender {
                sender_name1: "".to_string(),
                sender_zip_code: "".to_string(),
                sender_address1: "".to_string(),
                sender_tel: "".to_string(),
                sender_email: "".to_string(),
                bank_accounts: banks,
            }
        };

        let body = Body {
            issue_date: input.issue_date.clone(),
            subject: input.subject.clone(),
            payment_due_on: input.payment_due_on.clone(),
            contact_id: input.contact_id.parse().unwrap(),
            items: vec![ItemBody {
                name: input.subject.clone(),
                quantity: 1,
                unit_price: input.billing_amount.clone(),
                tax_type: "STANDARD_TAX_10".to_string(),
                excluding_withholding_tax: false,
            }],
            body: sender,
        };

        println!("json body: {}", serde_json::to_string(&body).unwrap());

        let query = vec![];

        self.call(
            CallInput {
                method: Method::POST,
                path: "/api/v3/invoice".to_string(),
                body: Some(
                    serde_json::to_string(&body)
                        .map_err(|e| CoreError::Internal(e.to_string()))?
                        .into(),
                ),
                query,
            },
            input.access_token.clone(),
        )
        .await?
        .error_for_status()?
        .json::<create_invoice::Output>()
        .await
        .map_err(CoreError::from)
        .map(|item| item.to_domain(input.supplier_id.clone()).unwrap())
    }
}

pub mod get_invoices {
    use super::*;

    pub struct Input {
        pub access_token: String,
        pub page: i32,
        pub per_page: i32,
        pub supplier_id: String,
        pub contact_group_id: String,
    }

    pub type Output = Vec<Invoice>;
}

pub mod get_pdf {
    use super::*;

    pub struct Input {
        pub access_token: String,
        pub invoice_id: String,
    }

    pub type Output = Bytes;
}

pub mod create_invoice {
    use super::*;
    use chrono::{DateTime, Utc};

    pub struct Input {
        pub access_token: String,
        pub supplier_id: String,
        pub contact_id: String,
        pub subject: String,
        pub issue_date: String,
        pub payment_due_on: String,
        pub billing_amount: i32,
        pub bank: Option<domain::bank::Bank>,
        pub sender: Option<domain::sender::Sender>,
        pub now: DateTime<Utc>,
    }

    pub type Output = Invoice;
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
    fn to_domain(&self, supplier_id: String) -> CoreResult<domain::invoice::Invoice> {
        let body = self.body.as_ref().unwrap();

        let total_amount: f64 = body
            .total_amount
            .clone()
            .unwrap_or("0".to_string())
            .parse()
            .unwrap();
        let tax: f64 = body.tax.clone().unwrap_or("0".to_string()).parse().unwrap();

        let issue_ymd = YMD::from_str(self.issue_date.clone().unwrap_or("".to_string()).as_str())
            .map_err(|_e| CoreError::Internal("cannot parse issue_date".to_string()))?;
        let payment_due_on_ymd = YMD::from_str(
            self.payment_due_on
                .clone()
                .unwrap_or("".to_string())
                .as_str(),
        )
        .map_err(|_e| CoreError::Internal("cannot parse payment_due_on".to_string()))?;

        let created_at =
            chrono::DateTime::parse_from_rfc3339(self.created_at.clone().unwrap().as_str())
                .map_err(|_e| CoreError::Internal("cannot parse created_at".to_string()))?;
        let updated_at =
            chrono::DateTime::parse_from_rfc3339(self.updated_at.clone().unwrap().as_str())
                .map_err(|_e| CoreError::Internal("cannot parse updated_at".to_string()))?;

        Ok(domain::invoice::Invoice {
            id: String::from(self.id.unwrap().to_string()),
            supplier_id,
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
