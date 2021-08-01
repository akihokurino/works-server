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
        pub id: String,
        pub contact_group_id: String,
        pub recipient_name: String,
    }
}
