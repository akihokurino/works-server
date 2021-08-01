use crate::misoca::{CallInput, Client};
use crate::{CoreError, CoreResult};
use reqwest::Method;
use serde::{Deserialize, Serialize};

impl Client {
    pub async fn get_contacts(
        &self,
        input: get_contacts::Input,
    ) -> CoreResult<get_contacts::Output> {
        #[derive(Debug, Serialize)]
        struct Body {}

        let body = Body {};

        println!("json body: {}", serde_json::to_string(&body).unwrap());

        let query = vec![
            ("page".to_string(), input.page.to_string()),
            ("per_page".to_string(), input.per_page.to_string()),
        ];

        self.call(
            CallInput {
                method: Method::GET,
                path: "/api/v3/contacts".to_string(),
                body: Some(
                    serde_json::to_string(&body)
                        .map_err(|e| CoreError::Internal(e.to_string()))?
                        .into(),
                ),
                query,
            },
            input.access_token,
        )
        .await?
        .error_for_status()?
        .json::<get_contacts::Output>()
        .await
        .map_err(CoreError::from)
    }

    pub async fn create_contact(
        &self,
        input: create_contact::Input,
    ) -> CoreResult<create_contact::Output> {
        #[derive(Debug, Serialize)]
        struct Body {
            pub recipient_name: String,
        }

        let body = Body {
            recipient_name: input.name,
        };

        println!("json body: {}", serde_json::to_string(&body).unwrap());

        let query = vec![];

        self.call(
            CallInput {
                method: Method::POST,
                path: "/api/v3/contact".to_string(),
                body: Some(
                    serde_json::to_string(&body)
                        .map_err(|e| CoreError::Internal(e.to_string()))?
                        .into(),
                ),
                query,
            },
            input.access_token,
        )
        .await?
        .error_for_status()?
        .json::<create_contact::Output>()
        .await
        .map_err(CoreError::from)
    }
}

pub mod get_contacts {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct Input {
        pub access_token: String,
        pub page: i32,
        pub per_page: i32,
    }

    pub type Output = Vec<Contact>;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Contact {
        pub id: Option<i32>,
        pub contact_group_id: Option<i32>,
        pub recipient_name: Option<String>,
    }
}

pub mod create_contact {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct Input {
        pub access_token: String,
        pub name: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Output {
        pub id: Option<i32>,
        pub contact_group_id: Option<i32>,
        pub recipient_name: Option<String>,
    }
}
